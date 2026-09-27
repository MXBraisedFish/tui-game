const assert = require('node:assert/strict');

const port = Number(process.argv[2]);
assert(Number.isInteger(port) && port > 0, 'missing DevTools port');

async function main() {
  const targets = await (await fetch(`http://127.0.0.1:${port}/json`)).json();
  const page = targets.find((target) => target.type === 'page' && target.url.startsWith('file:'));
  assert(page, 'local architecture page was not opened');

  const socket = new WebSocket(page.webSocketDebuggerUrl);
  const timeout = setTimeout(() => { console.error('Browser check timed out'); process.exit(1); }, 30000);
  const pending = new Map();
  const runtimeErrors = [];
  let nextId = 0;

  socket.addEventListener('message', (event) => {
    const message = JSON.parse(event.data);
    if (message.method === 'Runtime.exceptionThrown') {
      runtimeErrors.push(message.params.exceptionDetails.text);
    }
    if (message.method === 'Runtime.consoleAPICalled' && message.params.type === 'error') {
      runtimeErrors.push(message.params.args.map((arg) => arg.value || arg.description || '').join(' '));
    }
    if (message.id && pending.has(message.id)) {
      const { resolve, reject } = pending.get(message.id);
      pending.delete(message.id);
      if (message.error) reject(new Error(message.error.message));
      else resolve(message.result);
    }
  });

  await new Promise((resolve, reject) => {
    socket.addEventListener('open', resolve, { once: true });
    socket.addEventListener('error', reject, { once: true });
  });

  function call(method, params = {}) {
    const id = ++nextId;
    return new Promise((resolve, reject) => {
      pending.set(id, { resolve, reject });
      socket.send(JSON.stringify({ id, method, params }));
    });
  }

  async function evaluate(expression) {
    const response = await call('Runtime.evaluate', {
      expression,
      awaitPromise: true,
      returnByValue: true,
      userGesture: true,
    });
    if (response.exceptionDetails) {
      throw new Error(response.exceptionDetails.text);
    }
    return response.result.value;
  }

  async function afterPaint(expression) {
    return evaluate(`new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(() => resolve(${expression}))))`);
  }

  await call('Page.enable');
  await call('Runtime.enable');
  await evaluate(`new Promise(resolve => document.readyState === 'complete' ? resolve() : window.addEventListener('load', resolve, { once: true }))`);

  const initial = await afterPaint(`({
    title: document.title,
    datasets: Object.keys(window.ARCH_DATA),
    historySelector: Boolean(document.querySelector('#snapshot')),
    units: window.ARCH_DATA.current.units.length,
    graphNodes: document.querySelectorAll('#graph-svg .node').length,
    active: document.querySelector('.view.active').id
  })`);
  assert.deepEqual(initial.datasets, ['current']);
  assert.equal(initial.historySelector, false);
  assert.equal(initial.active, 'view-graph');
  assert.equal(initial.graphNodes, initial.units);
  assert(initial.units > 0);

  await evaluate(`document.querySelector('[data-view="flow"]').click()`);
  const flow = await afterPaint(`({ active: document.querySelector('.view.active').id, nodes: document.querySelectorAll('#flow-svg .node').length })`);
  assert.equal(flow.active, 'view-flow');
  assert(flow.nodes > 0, 'flow view did not render nodes');

  await evaluate(`document.querySelector('[data-view="tree"]').click()`);
  const tree = await afterPaint(`({ active: document.querySelector('.view.active').id, svgGroups: document.querySelectorAll('#tree-svg g').length })`);
  assert.equal(tree.active, 'view-tree');
  assert(tree.svgGroups > 0, 'tree view did not render');

  await evaluate(`document.querySelector('[data-view="graph"]').click()`);
  const graph = await afterPaint(`({ active: document.querySelector('.view.active').id, nodes: document.querySelectorAll('#graph-svg .node').length })`);
  assert.equal(graph.active, 'view-graph');
  assert.equal(graph.nodes, initial.units);

  const search = await evaluate(`(() => {
    const input = document.querySelector('#graph-search');
    input.value = 'tg_service_video';
    input.dispatchEvent(new Event('input', { bubbles: true }));
    const row = document.querySelector('#graph-search-results [data-id]');
    if (!row) return { found: false };
    const id = row.dataset.id;
    input.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }));
    return { found: true, id };
  })()`);
  assert.equal(search.found, true);
  assert.equal(search.id, 'service/tg_service_video');
  const focused = await afterPaint(`({
    active: document.querySelector('.view.active').id,
    panel: document.querySelector('#panel').textContent,
    searchValue: document.querySelector('#graph-search').value,
    focusNode: document.querySelector('#graph-svg .node.selected') !== null
  })`);
  assert.equal(focused.active, 'view-graph');
  assert(focused.panel.includes(search.id), 'search did not locate the selected package');
  assert.equal(focused.searchValue, '');
  assert.equal(focused.focusNode, true);

  await evaluate(`document.querySelector('[data-view="findings"]').click()`);
  const findings = await afterPaint(`({ active: document.querySelector('.view.active').id, content: document.querySelector('#findings').textContent })`);
  assert.equal(findings.active, 'view-findings');
  assert(findings.content.includes('当前数据未附人工审计记录'));

  await evaluate(`document.querySelector('[data-view="graph"]').click(); document.querySelector('[data-action="overview"]').click()`);
  await evaluate(`document.querySelector('#panel-toggle').click()`);
  assert.equal(await afterPaint(`getComputedStyle(document.querySelector('#inspector')).display`), 'none');
  await evaluate(`document.querySelector('#panel-toggle').click()`);
  assert.notEqual(await afterPaint(`getComputedStyle(document.querySelector('#inspector')).display`), 'none');

  const layouts = [];
  for (const [width, height] of [[1440, 900], [390, 844]]) {
    await call('Emulation.setDeviceMetricsOverride', { width, height, deviceScaleFactor: 1, mobile: false });
    const layout = await afterPaint(`(() => {
      const graph = document.querySelector('#view-graph').getBoundingClientRect();
      const panel = document.querySelector('#inspector').getBoundingClientRect();
      return { width: innerWidth, overflow: document.documentElement.scrollWidth > innerWidth,
        graphHeight: graph.height, separate: panel.left >= graph.right || panel.top >= graph.bottom };
    })()`);
    assert.equal(layout.overflow, false, 'page overflows viewport');
    assert.equal(layout.separate, true, 'inspector overlaps graph');
    assert(layout.graphHeight >= 160);
    layouts.push(layout);
    if (process.argv[3]) {
      const fs = require('node:fs');
      const path = require('node:path');
      fs.mkdirSync(process.argv[3], { recursive: true });
      const screenshot = await call('Page.captureScreenshot', { format: 'png' });
      fs.writeFileSync(path.join(process.argv[3], `architecture-${width}.png`), Buffer.from(screenshot.data, 'base64'));
    }
  }
  await call('Emulation.clearDeviceMetricsOverride');
  assert.deepEqual(runtimeErrors, []);
  console.log(JSON.stringify({ initial, flow, tree, graph, search, focused, layouts, runtimeErrors }, null, 2));
  clearTimeout(timeout);
  socket.close();
}

main().catch((error) => {
  console.error(error.stack || error);
  process.exitCode = 1;
});
