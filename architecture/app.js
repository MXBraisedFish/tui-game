(() => {
  'use strict';

  const SVG_NS = 'http://www.w3.org/2000/svg';
  const ALL = window.ARCH_DATA || {};
  const LAYER_COLOR = {
    main_loop: '#f5a524', ui: '#a78bfa', service: '#2dd4bf', core: '#fb7185',
  };
  const LAYER_NAME = { main_loop: '主循环', ui: '界面', service: '服务', core: '核' };
  const GROUP_COLOR = {
    常量: '#facc15', 接口: '#fb923c', 类型: '#60a5fa', 函数: '#4ade80', 方法: '#94a3b8', 其他: '#94a3b8',
  };
  const KIND_NAME = {
    const: '常量', static: '静态变量', assoc_const: '关联常量', trait: 'trait 接口', trait_fn: 'trait 方法',
    struct: '结构体', enum: '枚举', type: '类型别名', union: '联合体', fn: '函数', method: '方法', macro: '宏',
  };
  const SEVERITY_NAME = { high: '高', medium: '中', low: '低' };
  const CATEGORY_NAME = { security: '安全', correctness: '正确性', architecture: '架构', quality: '质量' };
  const SEVERITY_COLOR = { high: '#ef4444', medium: '#f59e0b', low: '#64748b' };

  let data = null;
  const state = { view: 'flow', graphMode: 'overview', focus: null, badOnly: false };
  const panel = document.getElementById('panel');

  // ---------- small helpers ----------

  function svgEl(tag, attrs, parent) {
    const node = document.createElementNS(SVG_NS, tag);
    for (const [key, value] of Object.entries(attrs || {})) {
      if (value !== undefined && value !== null) node.setAttribute(key, value);
    }
    if (parent) parent.appendChild(node);
    return node;
  }

  function svgText(parent, x, y, text, attrs) {
    const node = svgEl('text', Object.assign({ x, y }, attrs || {}), parent);
    node.textContent = text;
    return node;
  }

  function escapeHtml(text) {
    return String(text ?? '').replace(/[&<>"']/g, (ch) => ({
      '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;',
    }[ch]));
  }

  const measureContext = document.createElement('canvas').getContext('2d');
  function textWidth(text, font) {
    measureContext.font = font;
    return measureContext.measureText(text).width;
  }

  function truncate(text, maxWidth, font) {
    text = String(text ?? '');
    if (textWidth(text, font) <= maxWidth) return text;
    let low = 0;
    let high = text.length;
    while (low < high) {
      const middle = Math.ceil((low + high) / 2);
      if (textWidth(text.slice(0, middle) + '…', font) <= maxWidth) low = middle;
      else high = middle - 1;
    }
    return text.slice(0, low) + '…';
  }

  function wrapLines(text, maxWidth, font, maxLines) {
    const lines = [];
    let current = '';
    for (const ch of String(text ?? '')) {
      if (textWidth(current + ch, font) > maxWidth) {
        lines.push(current);
        current = ch;
        if (lines.length === maxLines) break;
      } else {
        current += ch;
      }
    }
    if (lines.length < maxLines && current) lines.push(current);
    if (lines.length === maxLines && lines.join('').length < String(text ?? '').length) {
      lines[maxLines - 1] = truncate(lines[maxLines - 1] + '…', maxWidth, font);
    }
    return lines;
  }

  function badge(text, color) {
    return `<span class="badge" style="background:${color}">${escapeHtml(text)}</span>`;
  }

  function unitById(id) {
    return data.units.find((unit) => unit.id === id);
  }

  function lastSegments(id) {
    const parts = id.split('/');
    if (parts.length <= 2) return id;
    return parts.slice(-2).join('/');
  }

  // ---------- pan & zoom ----------

  class PanZoom {
    constructor(svg) {
      this.svg = svg;
      this.x = 0;
      this.y = 0;
      this.k = 1;
      this.root = svgEl('g', {}, svg);
      this.dragging = null;
      svg.addEventListener('wheel', (event) => {
        event.preventDefault();
        const rect = svg.getBoundingClientRect();
        const factor = Math.exp(-event.deltaY * 0.0015);
        this.zoomAt(factor, event.clientX - rect.left, event.clientY - rect.top);
      }, { passive: false });
      svg.addEventListener('pointerdown', (event) => {
        if (event.target.closest('.node, .draggable')) return;
        this.dragging = { x: event.clientX, y: event.clientY, ox: this.x, oy: this.y };
        svg.classList.add('panning');
        svg.setPointerCapture(event.pointerId);
      });
      svg.addEventListener('pointermove', (event) => {
        if (!this.dragging) return;
        this.x = this.dragging.ox + event.clientX - this.dragging.x;
        this.y = this.dragging.oy + event.clientY - this.dragging.y;
        this.apply();
      });
      const stop = () => {
        this.dragging = null;
        svg.classList.remove('panning');
      };
      svg.addEventListener('pointerup', stop);
      svg.addEventListener('pointercancel', stop);
    }

    reset() {
      while (this.root.firstChild) this.root.removeChild(this.root.firstChild);
    }

    apply() {
      this.root.setAttribute('transform', `translate(${this.x},${this.y}) scale(${this.k})`);
    }

    zoomAt(factor, cx, cy) {
      const next = Math.min(4, Math.max(0.05, this.k * factor));
      const ratio = next / this.k;
      this.x = cx - (cx - this.x) * ratio;
      this.y = cy - (cy - this.y) * ratio;
      this.k = next;
      this.apply();
    }

    zoomCenter(factor) {
      const rect = this.svg.getBoundingClientRect();
      this.zoomAt(factor, rect.width / 2, rect.height / 2);
    }

    fit(padding = 40, maxScale = 1.2) {
      const box = this.root.getBBox();
      const rect = this.svg.getBoundingClientRect();
      if (!box.width || !box.height || !rect.width) return;
      const scale = Math.min(maxScale, (rect.width - padding * 2) / box.width, (rect.height - padding * 2) / box.height);
      this.k = Math.max(0.05, scale);
      this.x = (rect.width - box.width * this.k) / 2 - box.x * this.k;
      this.y = padding - box.y * this.k;
      if (box.height * this.k < rect.height - padding * 2) {
        this.y = (rect.height - box.height * this.k) / 2 - box.y * this.k;
      }
      this.apply();
    }

    focusTop(x, y, scale) {
      const rect = this.svg.getBoundingClientRect();
      this.k = scale;
      this.x = rect.width / 2 - x * scale;
      this.y = 40 - y * scale;
      this.apply();
    }

    toLocal(clientX, clientY) {
      const rect = this.svg.getBoundingClientRect();
      return { x: (clientX - rect.left - this.x) / this.k, y: (clientY - rect.top - this.y) / this.k };
    }
  }

  function addArrowMarker(svg, id, color) {
    let defs = svg.querySelector('defs');
    if (!defs) defs = svgEl('defs', {}, svg);
    const marker = svgEl('marker', {
      id, viewBox: '0 0 10 10', refX: 9, refY: 5, markerWidth: 7, markerHeight: 7, orient: 'auto-start-reverse',
    }, defs);
    svgEl('path', { d: 'M 0 0 L 10 5 L 0 10 z', fill: color }, marker);
  }

  // Makes an SVG group draggable in the pan/zoom space. `onMove` receives the
  // new top-left position; `onClick` fires when the pointer barely moved.
  function makeDraggable(group, zoom, getPosition, onMove, onClick) {
    group.classList.add('draggable');
    group.addEventListener('pointerdown', (event) => {
      event.stopPropagation();
      const start = zoom.toLocal(event.clientX, event.clientY);
      const origin = getPosition();
      let moved = false;
      group.setPointerCapture(event.pointerId);
      const move = (moveEvent) => {
        const point = zoom.toLocal(moveEvent.clientX, moveEvent.clientY);
        const dx = point.x - start.x;
        const dy = point.y - start.y;
        if (!moved && Math.hypot(dx, dy) * zoom.k < 4) return;
        moved = true;
        onMove(origin.x + dx, origin.y + dy);
      };
      const up = () => {
        group.removeEventListener('pointermove', move);
        group.removeEventListener('pointerup', up);
        group.removeEventListener('pointercancel', up);
        if (!moved && onClick) onClick();
      };
      group.addEventListener('pointermove', move);
      group.addEventListener('pointerup', up);
      group.addEventListener('pointercancel', up);
    });
  }

  function bezier(sx, sy, tx, ty) {
    const dx = Math.max(50, Math.abs(tx - sx) * 0.5);
    return `M ${sx} ${sy} C ${sx + dx} ${sy}, ${tx - dx} ${ty}, ${tx} ${ty}`;
  }

  // ---------- panel ----------

  function setPanel(html) {
    panel.innerHTML = html;
    panel.scrollTop = 0;
    panel.querySelectorAll('[data-unit]').forEach((node) => {
      node.addEventListener('click', () => focusUnit(node.dataset.unit));
    });
  }

  function unitPanelHtml(unit) {
    const deps = Object.entries(unit.deps).sort((a, b) => b[1] - a[1]);
    const dependents = Object.entries(unit.dependents).sort((a, b) => b[1] - a[1]);
    const counts = {};
    unit.items.forEach((item) => { counts[item.g] = (counts[item.g] || 0) + 1; });
    const countText = Object.entries(counts).map(([group, count]) => `${group} ${count}`).join(' · ') || '无';
    const unitChecks = data.checks.filter((check) => (check.type === 'layer' && check.from === unit.id)
      || (check.type === 'cycle' && check.members.includes(unit.id)));
    const unitFindings = data.findings.filter((finding) => unit.files.some((file) => file.path === finding.file));
    return `
      <h2>${escapeHtml(unit.id)}</h2>
      <div>${badge(LAYER_NAME[unit.layer], LAYER_COLOR[unit.layer])}
        <span class="muted">${unit.lines} 行 · ${unit.files.length} 个文件 · ${unit.items.length} 个条目</span></div>
      <p>${escapeHtml(unit.summary || '（暂无概述）')}</p>
      ${unit.notes ? `<p class="muted">${escapeHtml(unit.notes)}</p>` : ''}
      <h3>条目分布</h3><p class="muted">${escapeHtml(countText)}</p>
      <h3>依赖（${deps.length}）</h3>
      <div class="dep-list">${deps.map(([id, count]) => `<span class="chip" data-unit="${escapeHtml(id)}" title="引用 ${count} 次">${escapeHtml(id)} <b>${count}</b></span>`).join('') || '<span class="empty">无</span>'}</div>
      <h3>被依赖（${dependents.length}）</h3>
      <div class="dep-list">${dependents.map(([id, count]) => `<span class="chip" data-unit="${escapeHtml(id)}" title="被引用 ${count} 次">${escapeHtml(id)} <b>${count}</b></span>`).join('') || '<span class="empty">无</span>'}</div>
      ${unitChecks.length ? `<h3>架构检查</h3><ul>${unitChecks.map((check) => `<li>${escapeHtml(check.rule)}${check.type === 'layer' ? `：→ ${escapeHtml(check.to)}` : `（${check.members.length} 个模块）`}</li>`).join('')}</ul>` : ''}
      ${unitFindings.length ? `<h3>审计发现（${unitFindings.length}）</h3><ul>${unitFindings.map((finding) => `<li>${badge(SEVERITY_NAME[finding.severity] || finding.severity, SEVERITY_COLOR[finding.severity] || '#64748b')} ${escapeHtml(finding.title)} <span class="ref">${escapeHtml(finding.file)}:${finding.line}</span></li>`).join('')}</ul>` : ''}
      <h3>文件</h3>
      <ul>${unit.files.map((file) => `<li class="ref">${escapeHtml(file.path)}（${file.lines} 行）</li>`).join('')}</ul>`;
  }

  function itemPanelHtml(unit, item) {
    return `
      <h2 class="mono">${escapeHtml(item.p ? item.p + '::' : '')}${escapeHtml(item.n)}</h2>
      <div>${badge(KIND_NAME[item.k] || item.k, GROUP_COLOR[item.g] || '#94a3b8')}
        <span class="muted">${escapeHtml(item.v || '私有')}${item.t ? ` · 实现 ${escapeHtml(item.t)}` : ''}</span></div>
      <p>${escapeHtml(item.d || '（暂无描述）')}</p>
      <p class="ref">${escapeHtml(item.f)}:${item.l}</p>
      <h3>所属模块</h3>
      <p><span class="linkish" data-unit="${escapeHtml(unit.id)}">${escapeHtml(unit.id)}</span></p>`;
  }

  function defaultPanel() {
    const hints = {
      flow: `<h2>主循环流程图</h2>
        <p>展示 <span class="ref">host_engine::run()</span> 从启动、运行到退出的完整流程，以及故障分支。</p>
        <p class="muted">点击任意节点查看说明和源码位置。拖动空白处平移，滚轮缩放。</p>`,
      tree: `<h2>模块树</h2>
        <p>按源码的 <span class="ref">mod</span> 声明展开的完整模块树，颜色表示所属层级。</p>
        <p class="muted">点击圆点展开或收起，点击名称查看模块概述与依赖。数字为该模块文件的行数。</p>`,
      graph: `<h2>依赖节点图</h2>
        <p><b>总览</b>：每张卡片是一个模块，连线表示依赖（从使用方指向被使用方）。鼠标悬停高亮相关连线，红色表示违反分层规则。</p>
        <p><b>点击模块</b>进入三层视图：左侧是依赖的模块，中间是当前模块，右侧是它实现的常量、接口、类型、函数和方法，每条都附一句说明。</p>
        <p class="muted">卡片可拖动；拖动空白处平移，滚轮缩放。</p>`,
      findings: `<h2>审计发现</h2>
        <p>上半部分是根据依赖数据自动检查出的架构问题；下半部分是通读代码时记录的安全、正确性、架构和质量问题。</p>`,
    };
    setPanel(hints[state.view] + statsHtml());
  }

  function statsHtml() {
    const stats = data.stats;
    return `<h3>快照：${escapeHtml(data.snapshot)}（生成于 ${escapeHtml(data.generatedAt)}）</h3>
      <ul class="muted">
        <li>${stats.files} 个源文件，${stats.lines} 行</li>
        <li>${stats.units} 个模块，${stats.edges} 条模块间依赖</li>
        <li>${stats.items} 个条目，其中 ${stats.described} 个有说明</li>
        <li>${data.checks.length} 条自动检查结果，${stats.findings} 条审计发现</li>
      </ul>`;
  }

  // ---------- flowchart ----------

  const flowSvg = document.getElementById('flow-svg');
  const flowZoom = new PanZoom(flowSvg);
  addArrowMarker(flowSvg, 'arrow', '#7c8199');
  addArrowMarker(flowSvg, 'arrow-fault', '#ef4444');
  addArrowMarker(flowSvg, 'arrow-loop', '#2dd4bf');

  const FLOW = { colWidth: 400, nodeWidth: 320, nodeHeight: 50, rowHeight: 72 };
  const FLOW_STYLE = {
    start: { fill: '#3a2a26', stroke: '#ff6d5a' },
    end: { fill: '#3a2a26', stroke: '#ff6d5a' },
    process: { fill: '#262833', stroke: '#4a4e66' },
    decision: { fill: '#302a1c', stroke: '#f5a524' },
    io: { fill: '#1f2a33', stroke: '#60a5fa' },
    async: { fill: '#1c2e2c', stroke: '#2dd4bf', dash: '5 4' },
    loop: { fill: '#1c2e2c', stroke: '#2dd4bf' },
    fault: { fill: '#3a1f24', stroke: '#ef4444' },
    note: { fill: '#302d1c', stroke: '#facc15', dash: '3 3' },
  };
  const FLOW_TYPE_NAME = {
    start: '开始', end: '结束', process: '处理步骤', decision: '判断', io: '终端 / 输入输出', async: '异步任务',
    loop: '循环', fault: '故障处理', note: '说明',
  };

  function flowBox(node) {
    const x = node.col * FLOW.colWidth;
    const y = node.row * FLOW.rowHeight;
    return { x, y, w: FLOW.nodeWidth, h: FLOW.nodeHeight, cx: x + FLOW.nodeWidth / 2, cy: y + FLOW.nodeHeight / 2 };
  }

  function flowShape(parent, node, box) {
    const style = FLOW_STYLE[node.type] || FLOW_STYLE.process;
    const common = { fill: style.fill, stroke: style.stroke, 'stroke-width': 1.5, 'stroke-dasharray': style.dash };
    if (node.type === 'decision') {
      const inset = 18;
      const points = [
        [box.x + inset, box.y], [box.x + box.w - inset, box.y], [box.x + box.w, box.cy],
        [box.x + box.w - inset, box.y + box.h], [box.x + inset, box.y + box.h], [box.x, box.cy],
      ].map((point) => point.join(',')).join(' ');
      return svgEl('polygon', Object.assign({ points, class: 'shape' }, common), parent);
    }
    if (node.type === 'io') {
      const skew = 12;
      const points = [
        [box.x + skew, box.y], [box.x + box.w, box.y], [box.x + box.w - skew, box.y + box.h], [box.x, box.y + box.h],
      ].map((point) => point.join(',')).join(' ');
      return svgEl('polygon', Object.assign({ points, class: 'shape' }, common), parent);
    }
    const radius = node.type === 'start' || node.type === 'end' ? box.h / 2 : 8;
    return svgEl('rect', Object.assign({ x: box.x, y: box.y, width: box.w, height: box.h, rx: radius, class: 'shape' }, common), parent);
  }

  function flowNodesBetween(nodes, col, rowA, rowB) {
    const low = Math.min(rowA, rowB);
    const high = Math.max(rowA, rowB);
    return nodes.some((node) => node.col === col && node.row > low && node.row < high);
  }

  function flowEdgePath(edge, source, target, nodes) {
    const a = flowBox(source);
    const b = flowBox(target);
    const lane = 26;
    if (edge.kind === 'loop' || (source.col === target.col && target.row < source.row)) {
      const x = a.x - lane - 8;
      return { d: `M ${a.x} ${a.cy} H ${x} V ${b.cy} H ${b.x}`, lx: x - 6, ly: (a.cy + b.cy) / 2, anchor: 'end' };
    }
    if (edge.kind === 'break') {
      const x = a.x + a.w + lane;
      const targetX = b.x + b.w;
      const routeX = Math.max(x, targetX + lane);
      return { d: `M ${a.x + a.w} ${a.cy} H ${routeX} V ${b.cy} H ${targetX}`, lx: routeX + 6, ly: (a.cy + b.cy) / 2, anchor: 'start' };
    }
    if (source.row === target.row) {
      const toRight = target.col > source.col;
      const sx = toRight ? a.x + a.w : a.x;
      const tx = toRight ? b.x : b.x + b.w;
      return { d: `M ${sx} ${a.cy} H ${tx}`, lx: (sx + tx) / 2, ly: a.cy - 6, anchor: 'middle' };
    }
    if (source.col === target.col) {
      if (!flowNodesBetween(nodes, source.col, source.row, target.row)) {
        return { d: `M ${a.cx} ${a.y + a.h} V ${b.y}`, lx: a.cx + 8, ly: a.y + a.h + 14, anchor: 'start' };
      }
      const leftSide = edge.kind === 'fault';
      const x = leftSide ? a.x - lane : a.x + a.w + lane;
      const sx = leftSide ? a.x : a.x + a.w;
      const tx = leftSide ? b.x : b.x + b.w;
      return { d: `M ${sx} ${a.cy} H ${x} V ${b.cy} H ${tx}`, lx: x + (leftSide ? -6 : 6), ly: (a.cy + b.cy) / 2, anchor: leftSide ? 'end' : 'start' };
    }
    if (target.row > source.row) {
      const toRight = target.col > source.col;
      const tx = toRight ? b.x : b.x + b.w;
      if (source.col === 0 || source.col === 2) {
        return { d: `M ${a.cx} ${a.y + a.h} V ${b.cy} H ${tx}`, lx: a.cx + 8, ly: a.y + a.h + 14, anchor: 'start' };
      }
      const sx = toRight ? a.x + a.w : a.x;
      return { d: `M ${sx} ${a.cy} H ${b.cx} V ${b.y}`, lx: sx, ly: a.cy - 6, anchor: 'middle' };
    }
    return { d: `M ${a.cx} ${a.y} V ${b.cy} H ${target.col > source.col ? b.x : b.x + b.w}`, lx: a.cx, ly: a.y - 6, anchor: 'middle' };
  }

  function renderFlow() {
    flowZoom.reset();
    const flow = data.flow;
    const root = flowZoom.root;
    const nodes = flow.nodes;
    const byId = Object.fromEntries(nodes.map((node) => [node.id, node]));

    const groupLayer = svgEl('g', {}, root);
    const edgeLayer = svgEl('g', {}, root);
    const nodeLayer = svgEl('g', {}, root);

    flow.groups.forEach((group, index) => {
      const pad = 22 + (group.id === 'g_frame' ? 0 : 14);
      const x = group.col[0] * FLOW.colWidth - pad;
      const y = group.row[0] * FLOW.rowHeight - pad - 22;
      const w = (group.col[1] - group.col[0]) * FLOW.colWidth + FLOW.nodeWidth + pad * 2;
      const h = (group.row[1] - group.row[0]) * FLOW.rowHeight + FLOW.nodeHeight + pad * 2 + 22;
      const box = svgEl('rect', { x, y, width: w, height: h, rx: 12, class: 'group-box' }, groupLayer);
      if (group.id === 'g_frame') {
        box.setAttribute('stroke', '#2dd4bf');
        box.setAttribute('fill', 'rgba(45,212,191,0.03)');
      }
      const title = svgText(groupLayer, x + 14, y + 22, group.title, { class: 'group-title' });
      title.style.cursor = 'pointer';
      title.addEventListener('click', () => setPanel(`<h2>${escapeHtml(group.title)}</h2><p class="ref">${escapeHtml(group.ref)}</p>`));
      void index;
    });

    flow.edges.forEach((edge) => {
      const source = byId[edge.from];
      const target = byId[edge.to];
      if (!source || !target) return;
      const route = flowEdgePath(edge, source, target, nodes);
      let stroke = '#7c8199';
      let marker = 'url(#arrow)';
      if (edge.kind === 'fault') { stroke = '#ef4444'; marker = 'url(#arrow-fault)'; }
      if (edge.kind === 'loop' || edge.kind === 'break') { stroke = '#2dd4bf'; marker = 'url(#arrow-loop)'; }
      svgEl('path', {
        d: route.d, fill: 'none', stroke, 'stroke-width': 1.6,
        'stroke-dasharray': edge.kind === 'note' ? '3 4' : (edge.kind === 'side' ? '6 4' : undefined),
        'marker-end': edge.kind === 'note' ? undefined : marker,
      }, edgeLayer);
      if (edge.label) {
        svgText(edgeLayer, route.lx, route.ly, edge.label, { class: 'edge-label', 'text-anchor': route.anchor });
      }
    });

    nodes.forEach((node) => {
      const box = flowBox(node);
      const group = svgEl('g', { class: 'node' }, nodeLayer);
      flowShape(group, node, box);
      const labelFont = '600 13px "Segoe UI", "Microsoft YaHei", sans-serif';
      const detailFont = '11px "Segoe UI", "Microsoft YaHei", sans-serif';
      svgText(group, box.x + 16, box.y + 21, truncate(node.label, box.w - 32, labelFont), { 'font-size': 13, 'font-weight': 600 });
      svgText(group, box.x + 16, box.y + 38, truncate(node.detail, box.w - 32, detailFont), { 'font-size': 11, class: 'dim' });
      const title = svgEl('title', {}, group);
      title.textContent = `${node.label}\n${node.detail}\n${node.ref}`;
      group.addEventListener('click', () => {
        nodeLayer.querySelectorAll('.node.selected').forEach((other) => other.classList.remove('selected'));
        group.classList.add('selected');
        setPanel(`<h2>${escapeHtml(node.label)}</h2>
          <div>${badge(FLOW_TYPE_NAME[node.type] || node.type, (FLOW_STYLE[node.type] || FLOW_STYLE.process).stroke)}</div>
          <p>${escapeHtml(node.detail)}</p><p class="ref">${escapeHtml(node.ref)}</p>`);
      });
    });

    document.getElementById('flow-legend').innerHTML = Object.entries(FLOW_TYPE_NAME)
      .map(([type, name]) => `<span><i style="background:${FLOW_STYLE[type].stroke}"></i>${name}</span>`).join('')
      + '<span><i style="background:#ef4444"></i>红线：故障路径</span><span><i style="background:#2dd4bf"></i>青线：循环 / 跳出</span>';
    requestAnimationFrame(() => flowZoom.focusTop(FLOW.colWidth * 1.5, -60, 0.85));
  }

  // ---------- module tree ----------

  const treeSvg = document.getElementById('tree-svg');
  const treeZoom = new PanZoom(treeSvg);
  const TREE = { dx: 230, dy: 24 };
  let treeRoot = null;

  function prepareTree(node, depth) {
    node._depth = depth;
    node._collapsed = depth >= 3 && Boolean(node.children);
    (node.children || []).forEach((child) => prepareTree(child, depth + 1));
  }

  function layoutTree(node, cursor) {
    node._x = node._depth * TREE.dx;
    const kids = node._collapsed ? [] : (node.children || []);
    if (!kids.length) {
      node._y = cursor.y;
      cursor.y += TREE.dy;
      return;
    }
    kids.forEach((kid) => layoutTree(kid, cursor));
    node._y = (kids[0]._y + kids[kids.length - 1]._y) / 2;
  }

  function renderTree(keepView) {
    const saved = { x: treeZoom.x, y: treeZoom.y, k: treeZoom.k };
    treeZoom.reset();
    layoutTree(treeRoot, { y: 0 });
    const linkLayer = svgEl('g', {}, treeZoom.root);
    const nodeLayer = svgEl('g', {}, treeZoom.root);

    function draw(node) {
      const kids = node._collapsed ? [] : (node.children || []);
      kids.forEach((kid) => {
        const sx = node._x + 6;
        const mid = (node._x + kid._x) / 2;
        svgEl('path', {
          d: `M ${sx} ${node._y} C ${mid} ${node._y}, ${mid} ${kid._y}, ${kid._x - 6} ${kid._y}`,
          fill: 'none', stroke: '#3f4358', 'stroke-width': 1.2,
        }, linkLayer);
        draw(kid);
      });
      const color = LAYER_COLOR[node.layer] || '#94a3b8';
      const group = svgEl('g', {}, nodeLayer);
      const hasChildren = Boolean(node.children && node.children.length);
      const dot = svgEl('circle', {
        cx: node._x, cy: node._y, r: hasChildren ? 6 : 4,
        fill: hasChildren && node._collapsed ? color : '#16171d', stroke: color, 'stroke-width': 2,
      }, group);
      dot.style.cursor = hasChildren ? 'pointer' : 'default';
      if (hasChildren) {
        dot.addEventListener('click', () => {
          node._collapsed = !node._collapsed;
          renderTree(true);
        });
      }
      const label = svgText(group, node._x + 12, node._y + 4, node.name, { 'font-size': 13, class: 'mono' });
      label.style.cursor = 'pointer';
      const labelWidth = textWidth(node.name, '13px Consolas, monospace');
      const size = node.children ? `${node.total} 行` : `${node.lines} 行`;
      svgText(group, node._x + 18 + labelWidth, node._y + 4, size + (node._collapsed && hasChildren ? ` · ${node.children.length} 子模块` : ''), { 'font-size': 11, class: 'faint' });
      label.addEventListener('click', () => {
        const unit = unitById(node.unit);
        const header = `<p class="ref">${escapeHtml(node.module)}</p>${node.file ? `<p class="ref">${escapeHtml(node.file)}（${node.lines} 行）</p>` : ''}`;
        if (unit) setPanel(header + unitPanelHtml(unit));
        else setPanel(`<h2>${escapeHtml(node.name)}</h2>${header}`);
      });
    }

    draw(treeRoot);
    document.getElementById('tree-legend').innerHTML = Object.entries(LAYER_NAME)
      .map(([layer, name]) => `<span><i style="background:${LAYER_COLOR[layer]}"></i>${name}</span>`).join('')
      + '<span>实心圆点：已收起，可展开</span>';
    if (keepView) {
      Object.assign(treeZoom, saved);
      treeZoom.apply();
    } else {
      requestAnimationFrame(() => treeZoom.fit(40, 1));
    }
  }

  function setTreeCollapsed(node, collapseFrom) {
    node._collapsed = collapseFrom !== null && node._depth >= collapseFrom && Boolean(node.children);
    (node.children || []).forEach((child) => setTreeCollapsed(child, collapseFrom));
  }

  // ---------- dependency graph ----------

  const graphSvg = document.getElementById('graph-svg');
  const graphZoom = new PanZoom(graphSvg);
  addArrowMarker(graphSvg, 'garrow', '#5b6078');
  const CARD = { w: 220, h: 56 };
  let overviewPositions = null;

  function badEdgeSet() {
    const bad = new Set();
    data.checks.forEach((check) => {
      if (check.type === 'layer') bad.add(`${check.from}|${check.to}`);
      if (check.type === 'cycle') {
        check.mutual.forEach(([a, b]) => {
          bad.add(`${a}|${b}`);
          bad.add(`${b}|${a}`);
        });
      }
    });
    return bad;
  }

  function overviewLayout() {
    const positions = {};
    const columns = [
      { layer: 'main_loop', perColumn: 14 },
      { layer: 'ui', perColumn: 16 },
      { layer: 'service', perColumn: 21 },
      { layer: 'core', perColumn: 14 },
    ];
    let columnX = 0;
    const headers = [];
    columns.forEach((column) => {
      const units = data.units.filter((unit) => unit.layer === column.layer).sort((a, b) => a.id.localeCompare(b.id));
      const columnCount = Math.max(1, Math.ceil(units.length / column.perColumn));
      headers.push({ layer: column.layer, x: columnX, w: columnCount * 270 - 50, count: units.length });
      units.forEach((unit, index) => {
        const col = Math.floor(index / column.perColumn);
        const row = index % column.perColumn;
        positions[unit.id] = { x: columnX + col * 270, y: row * 78 };
      });
      columnX += columnCount * 270 + 110;
    });
    return { positions, headers };
  }

  function renderGraph() {
    if (state.graphMode === 'focus' && state.focus && unitById(state.focus)) renderFocus(state.focus);
    else renderOverview();
  }

  function renderOverview() {
    state.graphMode = 'overview';
    graphZoom.reset();
    document.getElementById('graph-mode-hint').textContent = '总览：点击模块卡片进入三层视图';
    if (!overviewPositions) overviewPositions = overviewLayout();
    const { positions, headers } = overviewPositions;
    const root = graphZoom.root;
    const headerLayer = svgEl('g', {}, root);
    const edgeLayer = svgEl('g', {}, root);
    const nodeLayer = svgEl('g', {}, root);
    const bad = badEdgeSet();

    headers.forEach((header) => {
      svgEl('rect', { x: header.x - 20, y: -76, width: header.w + 40, height: 44, rx: 8, fill: 'rgba(255,255,255,0.03)', stroke: LAYER_COLOR[header.layer], 'stroke-opacity': 0.5 }, headerLayer);
      svgText(headerLayer, header.x, -48, `${LAYER_NAME[header.layer]}（${header.count}）`, { 'font-size': 16, 'font-weight': 700, fill: LAYER_COLOR[header.layer] });
    });

    const edgeNodes = [];
    data.edges.forEach((edge) => {
      const a = positions[edge.from];
      const b = positions[edge.to];
      if (!a || !b) return;
      const isBad = bad.has(`${edge.from}|${edge.to}`);
      const path = svgEl('path', {
        d: bezier(a.x + CARD.w, a.y + CARD.h / 2, b.x, b.y + CARD.h / 2),
        class: 'edge' + (isBad ? ' bad' : ''),
        'stroke-width': Math.min(4, 0.8 + Math.log2(edge.count + 1) * 0.5),
      }, edgeLayer);
      path.style.opacity = state.badOnly ? (isBad ? 0.9 : 0) : (isBad ? 0.55 : 0.13);
      edgeNodes.push({ edge, path, isBad });
    });

    function refreshEdges(unitId) {
      edgeNodes.forEach(({ edge, path }) => {
        if (edge.from !== unitId && edge.to !== unitId) return;
        const a = positions[edge.from];
        const b = positions[edge.to];
        path.setAttribute('d', bezier(a.x + CARD.w, a.y + CARD.h / 2, b.x, b.y + CARD.h / 2));
      });
    }

    function highlight(unitId) {
      const related = new Set([unitId]);
      edgeNodes.forEach(({ edge, path, isBad }) => {
        const hot = unitId && (edge.from === unitId || edge.to === unitId);
        if (hot) {
          related.add(edge.from);
          related.add(edge.to);
        }
        path.classList.toggle('hot', Boolean(hot) && !isBad);
        if (!unitId) path.style.opacity = state.badOnly ? (isBad ? 0.9 : 0) : (isBad ? 0.55 : 0.13);
        else path.style.opacity = hot ? 0.95 : 0.03;
      });
      nodeLayer.querySelectorAll('.node').forEach((node) => {
        node.style.opacity = !unitId || related.has(node.dataset.unit) ? 1 : 0.25;
      });
    }

    data.units.forEach((unit) => {
      const position = positions[unit.id];
      const group = svgEl('g', { class: 'node', 'data-unit': unit.id, transform: `translate(${position.x},${position.y})` }, nodeLayer);
      drawUnitCard(group, unit, CARD.w, CARD.h);
      group.addEventListener('mouseenter', () => highlight(unit.id));
      group.addEventListener('mouseleave', () => highlight(null));
      makeDraggable(group, graphZoom, () => positions[unit.id], (x, y) => {
        positions[unit.id] = { x, y };
        group.setAttribute('transform', `translate(${x},${y})`);
        refreshEdges(unit.id);
      }, () => focusUnit(unit.id));
    });

    document.getElementById('graph-legend').innerHTML = Object.entries(LAYER_NAME)
      .map(([layer, name]) => `<span><i style="background:${LAYER_COLOR[layer]}"></i>${name}</span>`).join('')
      + '<span><i style="background:#ef4444"></i>违规依赖（核的依赖、服务间双向依赖）</span><span>连线从使用方右侧指向被依赖方左侧</span>';
    requestAnimationFrame(() => graphZoom.fit(40, 0.9));
    defaultPanel();
  }

  function drawUnitCard(group, unit, width, height) {
    const color = LAYER_COLOR[unit.layer];
    svgEl('rect', { x: 0, y: 0, width, height, rx: 8, class: 'node-card' }, group);
    svgEl('rect', { x: 0, y: 0, width: 5, height, rx: 2, fill: color }, group);
    const nameFont = '600 13px Consolas, monospace';
    svgText(group, 14, 22, truncate(lastSegments(unit.id), width - 24, nameFont), { 'font-size': 13, 'font-weight': 600, class: 'mono' });
    const deps = Object.keys(unit.deps).length;
    const dependents = Object.keys(unit.dependents).length;
    svgText(group, 14, 41, `${unit.lines} 行 · 依赖 ${deps} · 被依赖 ${dependents}`, { 'font-size': 11, class: 'dim' });
    svgEl('circle', { cx: 0, cy: height / 2, r: 4, class: 'port' }, group);
    svgEl('circle', { cx: width, cy: height / 2, r: 4, class: 'port' }, group);
    const title = svgEl('title', {}, group);
    title.textContent = `${unit.id}\n${unit.summary || ''}`;
  }

  function groupItems(unit) {
    const order = ['常量', '接口', '类型', '函数'];
    const groups = order.map((name) => ({ name, title: name, items: [] }));
    const methods = new Map();
    unit.items.forEach((item) => {
      if (item.g === '方法') {
        const key = item.p || '?';
        if (!methods.has(key)) methods.set(key, []);
        methods.get(key).push(item);
        return;
      }
      if (item.k === 'trait_fn') {
        const key = `trait ${item.p}`;
        if (!methods.has(key)) methods.set(key, []);
        methods.get(key).push(item);
        return;
      }
      const target = groups.find((group) => group.name === item.g) || groups[3];
      target.items.push(item);
    });
    const result = groups.filter((group) => group.items.length);
    [...methods.entries()].sort((a, b) => a[0].localeCompare(b[0])).forEach(([owner, items]) => {
      const traitNames = [...new Set(items.map((item) => item.t).filter(Boolean))];
      result.push({
        name: '方法',
        title: `${owner} 的方法${traitNames.length ? `（含 ${traitNames.join('、')} 实现）` : ''}`,
        items,
      });
    });
    return result;
  }

  function renderFocus(unitId) {
    const unit = unitById(unitId);
    state.graphMode = 'focus';
    state.focus = unitId;
    graphZoom.reset();
    document.getElementById('graph-mode-hint').textContent = `三层视图：${unitId}（点击左侧依赖可跳转）`;
    const root = graphZoom.root;
    const edgeLayer = svgEl('g', {}, root);
    const nodeLayer = svgEl('g', {}, root);

    const deps = Object.entries(unit.deps).sort((a, b) => b[1] - a[1]);
    const groups = groupItems(unit);
    const columns = { deps: 0, unit: 380, items: 800 };
    const ROW = 24;
    const ITEM_W = 640;

    svgText(nodeLayer, columns.deps, -30, `第一层：依赖（${deps.length}）`, { 'font-size': 14, 'font-weight': 700, class: 'dim' });
    svgText(nodeLayer, columns.unit, -30, '第二层：当前模块', { 'font-size': 14, 'font-weight': 700, class: 'dim' });
    svgText(nodeLayer, columns.items, -30, `第三层：实现的条目（${unit.items.length}）`, { 'font-size': 14, 'font-weight': 700, class: 'dim' });

    const itemHeights = groups.map((group) => 40 + group.items.length * ROW + 8);
    const itemsTotal = itemHeights.reduce((sum, value) => sum + value + 18, 0);
    const depsTotal = deps.length * 62;
    const unitHeight = 150;
    const unitY = Math.max(0, Math.min(Math.max(itemsTotal, depsTotal) / 2 - unitHeight / 2, 400));

    const positions = { unit: { x: columns.unit, y: unitY } };
    const edgePaths = [];

    function redrawEdges() {
      edgePaths.forEach(({ path, from, to }) => {
        const a = from();
        const b = to();
        path.setAttribute('d', bezier(a.x, a.y, b.x, b.y));
      });
    }

    // Layer 2: the unit itself.
    const unitGroup = svgEl('g', { class: 'node selected', transform: `translate(${columns.unit},${unitY})` }, nodeLayer);
    const unitWidth = 300;
    svgEl('rect', { x: 0, y: 0, width: unitWidth, height: unitHeight, rx: 10, class: 'node-card' }, unitGroup);
    svgEl('rect', { x: 0, y: 0, width: unitWidth, height: 34, rx: 10, fill: LAYER_COLOR[unit.layer], opacity: 0.9 }, unitGroup);
    svgEl('rect', { x: 0, y: 24, width: unitWidth, height: 10, fill: LAYER_COLOR[unit.layer], opacity: 0.9 }, unitGroup);
    svgText(unitGroup, 14, 23, truncate(unit.id, unitWidth - 28, '700 14px Consolas, monospace'), { 'font-size': 14, 'font-weight': 700, class: 'mono', fill: '#111' });
    svgText(unitGroup, 14, 54, `${LAYER_NAME[unit.layer]} · ${unit.lines} 行 · ${unit.files.length} 个文件`, { 'font-size': 12, class: 'dim' });
    wrapLines(unit.summary || '（暂无概述）', unitWidth - 28, '12px "Microsoft YaHei", sans-serif', 4).forEach((line, index) => {
      svgText(unitGroup, 14, 76 + index * 17, line, { 'font-size': 12 });
    });
    svgEl('circle', { cx: 0, cy: unitHeight / 2, r: 5, class: 'port' }, unitGroup);
    svgEl('circle', { cx: unitWidth, cy: unitHeight / 2, r: 5, class: 'port' }, unitGroup);
    makeDraggable(unitGroup, graphZoom, () => positions.unit, (x, y) => {
      positions.unit = { x, y };
      unitGroup.setAttribute('transform', `translate(${x},${y})`);
      redrawEdges();
    }, () => setPanel(unitPanelHtml(unit)));

    // Layer 1: dependencies.
    deps.forEach(([depId, count], index) => {
      const dep = unitById(depId);
      const key = `dep:${depId}`;
      positions[key] = { x: columns.deps, y: index * 62 };
      const group = svgEl('g', { class: 'node', transform: `translate(${positions[key].x},${positions[key].y})` }, nodeLayer);
      const width = 250;
      svgEl('rect', { x: 0, y: 0, width, height: 46, rx: 8, class: 'node-card' }, group);
      svgEl('rect', { x: 0, y: 0, width: 5, height: 46, rx: 2, fill: LAYER_COLOR[dep ? dep.layer : 'service'] }, group);
      svgText(group, 14, 20, truncate(depId, width - 24, '600 12px Consolas, monospace'), { 'font-size': 12, 'font-weight': 600, class: 'mono' });
      const edge = data.edges.find((candidate) => candidate.from === unit.id && candidate.to === depId);
      const used = edge ? edge.items.join('、') : '';
      svgText(group, 14, 37, truncate(`引用 ${count} 次：${used}`, width - 24, '11px "Microsoft YaHei", sans-serif'), { 'font-size': 11, class: 'dim' });
      svgEl('circle', { cx: width, cy: 23, r: 4, class: 'port' }, group);
      const title = svgEl('title', {}, group);
      title.textContent = `${depId}\n使用了：${used}`;
      const path = svgEl('path', { class: 'edge', 'marker-end': 'url(#garrow)' }, edgeLayer);
      edgePaths.push({
        path,
        from: () => ({ x: positions[key].x + width, y: positions[key].y + 23 }),
        to: () => ({ x: positions.unit.x, y: positions.unit.y + unitHeight / 2 }),
      });
      makeDraggable(group, graphZoom, () => positions[key], (x, y) => {
        positions[key] = { x, y };
        group.setAttribute('transform', `translate(${x},${y})`);
        redrawEdges();
      }, () => focusUnit(depId));
    });

    // Layer 3: implemented items, grouped like n8n nodes with row lists.
    let cursorY = 0;
    groups.forEach((group, groupIndex) => {
      const key = `group:${groupIndex}`;
      const height = itemHeights[groupIndex];
      positions[key] = { x: columns.items, y: cursorY };
      cursorY += height + 18;
      const container = svgEl('g', { transform: `translate(${positions[key].x},${positions[key].y})` }, nodeLayer);
      const color = GROUP_COLOR[group.name] || '#94a3b8';
      svgEl('rect', { x: 0, y: 0, width: ITEM_W, height, rx: 8, class: 'node-card' }, container);
      const header = svgEl('g', { class: 'draggable' }, container);
      svgEl('rect', { x: 0, y: 0, width: ITEM_W, height: 32, rx: 8, fill: color, opacity: 0.18 }, header);
      svgEl('rect', { x: 0, y: 0, width: 5, height, rx: 2, fill: color }, container);
      svgText(header, 14, 21, `${group.title}（${group.items.length}）`, { 'font-size': 13, 'font-weight': 700, fill: color });
      svgEl('circle', { cx: 0, cy: 16, r: 4.5, class: 'port' }, container);
      group.items.forEach((item, index) => {
        const y = 40 + index * ROW;
        const row = svgEl('g', { class: 'node' }, container);
        svgEl('rect', { x: 6, y: y - 2, width: ITEM_W - 12, height: ROW - 2, rx: 4, fill: 'transparent' }, row)
          .classList.add('row-bg');
        const label = item.k === 'method' || item.k === 'trait_fn' ? `${item.n}()` : (item.k === 'fn' ? `${item.n}()` : item.n);
        const nameFont = '12px Consolas, monospace';
        svgText(row, 16, y + 14, truncate(label, 200, nameFont), { 'font-size': 12, class: 'mono', fill: item.v ? '#e6e7ee' : '#9a9db0' });
        svgText(row, 226, y + 14, truncate(item.d || '（暂无描述）', ITEM_W - 244, '12px "Microsoft YaHei", sans-serif'), { 'font-size': 12, class: item.d ? 'dim' : 'faint' });
        const title = svgEl('title', {}, row);
        title.textContent = `${KIND_NAME[item.k] || item.k} ${item.p ? item.p + '::' : ''}${item.n}\n${item.d || ''}\n${item.f}:${item.l}`;
        row.addEventListener('click', () => setPanel(itemPanelHtml(unit, item)));
        row.addEventListener('mouseenter', () => row.querySelector('.row-bg').setAttribute('fill', 'rgba(255,255,255,0.05)'));
        row.addEventListener('mouseleave', () => row.querySelector('.row-bg').setAttribute('fill', 'transparent'));
      });
      const path = svgEl('path', { class: 'edge', 'marker-end': 'url(#garrow)' }, edgeLayer);
      path.style.stroke = color;
      path.style.opacity = 0.7;
      edgePaths.push({
        path,
        from: () => ({ x: positions.unit.x + unitWidth, y: positions.unit.y + unitHeight / 2 }),
        to: () => ({ x: positions[key].x, y: positions[key].y + 16 }),
      });
      makeDraggable(header, graphZoom, () => positions[key], (x, y) => {
        positions[key] = { x, y };
        container.setAttribute('transform', `translate(${x},${y})`);
        redrawEdges();
      }, null);
    });
    redrawEdges();

    document.getElementById('graph-legend').innerHTML = Object.entries(GROUP_COLOR).slice(0, 5)
      .map(([name, color]) => `<span><i style="background:${color}"></i>${name}</span>`).join('')
      + '<span>白色名称：公开条目；灰色名称：私有条目</span><span>点击条目查看详情，点击左侧依赖跳转</span>';
    setPanel(unitPanelHtml(unit));
    requestAnimationFrame(() => {
      const rect = graphSvg.getBoundingClientRect();
      const scale = Math.min(1, rect.width / 1500);
      graphZoom.focusTop(columns.unit + 150, -70, scale);
    });
  }

  function focusUnit(unitId) {
    if (!unitById(unitId)) return;
    state.focus = unitId;
    state.graphMode = 'focus';
    switchView('graph');
  }

  // ---------- findings ----------

  function renderFindings() {
    const container = document.getElementById('findings');
    const layerChecks = data.checks.filter((check) => check.type === 'layer');
    const cycleChecks = data.checks.filter((check) => check.type === 'cycle');
    const findings = data.findings.slice().sort((a, b) => ['high', 'medium', 'low'].indexOf(a.severity) - ['high', 'medium', 'low'].indexOf(b.severity));
    const bySeverity = {};
    const byCategory = {};
    findings.forEach((finding) => {
      bySeverity[finding.severity] = (bySeverity[finding.severity] || 0) + 1;
      byCategory[finding.category] = (byCategory[finding.category] || 0) + 1;
    });

    container.innerHTML = `
      <h2>概览</h2>
      <div class="cards">
        <div class="card"><div class="num">${layerChecks.length}</div><div class="lbl">分层违规依赖</div></div>
        <div class="card"><div class="num">${cycleChecks.length}</div><div class="lbl">服务循环依赖分量</div></div>
        ${['high', 'medium', 'low'].map((severity) => `<div class="card"><div class="num" style="color:${SEVERITY_COLOR[severity]}">${bySeverity[severity] || 0}</div><div class="lbl">${SEVERITY_NAME[severity]}风险发现</div></div>`).join('')}
        ${Object.entries(CATEGORY_NAME).map(([category, name]) => `<div class="card"><div class="num">${byCategory[category] || 0}</div><div class="lbl">${name}</div></div>`).join('')}
      </div>

      <h2>自动检查：分层违规</h2>
      <table class="list"><thead><tr><th>使用方 → 被依赖方</th><th>违反的规则</th><th>引用的条目</th><th>位置</th></tr></thead><tbody>
      ${layerChecks.map((check) => `<tr>
        <td><span class="linkish" data-unit="${escapeHtml(check.from)}">${escapeHtml(check.from)}</span> → <span class="linkish" data-unit="${escapeHtml(check.to)}">${escapeHtml(check.to)}</span></td>
        <td>${escapeHtml(check.rule)}</td>
        <td class="ref">${escapeHtml(check.items.join(', '))}</td>
        <td class="ref">${check.sites.slice(0, 3).map(escapeHtml).join('<br>')}</td></tr>`).join('') || '<tr><td colspan="4" class="muted">无</td></tr>'}
      </tbody></table>

      <h2>自动检查：服务之间的循环依赖</h2>
      ${cycleChecks.map((check) => `<div class="cycle-box">
        <div><b>${check.members.length} 个服务互相可达</b>，内部依赖边 ${check.edges} 条。${escapeHtml(check.rule)}</div>
        <div class="members">${check.members.map((member) => `<span class="chip linkish" data-unit="${escapeHtml(member)}">${escapeHtml(member)}</span>`).join('')}</div>
        <div class="muted">直接双向依赖的模块对（${check.mutual.length}）：${check.mutual.map(([a, b]) => `${escapeHtml(a)} ⇄ ${escapeHtml(b)}`).join('；') || '无（通过更长的环路形成循环）'}</div>
      </div>`).join('') || '<p class="muted">无</p>'}

      <h2>通读代码的审计发现（${findings.length}）</h2>
      <div class="filters">
        <select id="f-severity"><option value="">全部风险等级</option>${['high', 'medium', 'low'].map((severity) => `<option value="${severity}">${SEVERITY_NAME[severity]}</option>`).join('')}</select>
        <select id="f-category"><option value="">全部类别</option>${Object.entries(CATEGORY_NAME).map(([category, name]) => `<option value="${category}">${name}</option>`).join('')}</select>
        <input type="search" id="f-text" placeholder="搜索标题、说明或文件" style="width:260px">
        <span class="muted" id="f-count"></span>
      </div>
      <table class="list"><thead><tr><th style="width:70px">等级</th><th style="width:70px">类别</th><th>问题</th><th>位置</th></tr></thead><tbody id="f-body"></tbody></table>`;

    const body = container.querySelector('#f-body');
    const severity = container.querySelector('#f-severity');
    const category = container.querySelector('#f-category');
    const search = container.querySelector('#f-text');
    const count = container.querySelector('#f-count');

    function refresh() {
      const text = search.value.trim().toLowerCase();
      const rows = findings.filter((finding) => (!severity.value || finding.severity === severity.value)
        && (!category.value || finding.category === category.value)
        && (!text || `${finding.title} ${finding.detail} ${finding.file}`.toLowerCase().includes(text)));
      count.textContent = `显示 ${rows.length} 条`;
      body.innerHTML = rows.map((finding) => `<tr>
        <td>${badge(SEVERITY_NAME[finding.severity] || finding.severity, SEVERITY_COLOR[finding.severity] || '#64748b')}</td>
        <td>${escapeHtml(CATEGORY_NAME[finding.category] || finding.category)}</td>
        <td><b>${escapeHtml(finding.title)}</b><div class="detail">${escapeHtml(finding.detail)}</div></td>
        <td class="ref">${escapeHtml(finding.file)}:${finding.line}</td></tr>`).join('') || '<tr><td colspan="4" class="muted">没有匹配的发现</td></tr>';
    }

    [severity, category].forEach((control) => control.addEventListener('change', refresh));
    search.addEventListener('input', refresh);
    refresh();
    container.querySelectorAll('[data-unit]').forEach((node) => {
      node.addEventListener('click', () => focusUnit(node.dataset.unit));
    });
  }

  // ---------- view switching ----------

  const rendered = {};

  function switchView(view) {
    state.view = view;
    document.querySelectorAll('.tab').forEach((tab) => tab.classList.toggle('active', tab.dataset.view === view));
    document.querySelectorAll('.view').forEach((section) => section.classList.toggle('active', section.id === `view-${view}`));
    if (view === 'graph') {
      renderGraph();
      rendered.graph = true;
      return;
    }
    if (!rendered[view]) {
      rendered[view] = true;
      if (view === 'flow') renderFlow();
      if (view === 'tree') renderTree(false);
      if (view === 'findings') renderFindings();
    }
    defaultPanel();
  }

  function loadSnapshot(name) {
    data = ALL[name];
    overviewPositions = null;
    treeRoot = data.tree;
    prepareTree(treeRoot, 0);
    Object.keys(rendered).forEach((key) => delete rendered[key]);
    const stats = data.stats;
    document.getElementById('stats').innerHTML = [
      `<span class="chip"><b>${stats.files}</b> 文件</span>`,
      `<span class="chip"><b>${stats.lines.toLocaleString()}</b> 行</span>`,
      `<span class="chip"><b>${stats.units}</b> 模块</span>`,
      `<span class="chip"><b>${stats.edges}</b> 依赖</span>`,
      `<span class="chip"><b>${stats.items}</b> 条目</span>`,
    ].join('');
    if (state.focus && !unitById(state.focus)) {
      state.focus = null;
      state.graphMode = 'overview';
    }
    switchView(state.view);
  }

  document.querySelectorAll('.tab').forEach((tab) => tab.addEventListener('click', () => {
    if (tab.dataset.view === 'graph' && state.view === 'graph') state.graphMode = 'overview';
    switchView(tab.dataset.view);
  }));

  document.querySelectorAll('#view-flow [data-action]').forEach((button) => button.addEventListener('click', () => {
    if (button.dataset.action === 'fit') flowZoom.fit(30, 1);
    if (button.dataset.action === 'zoom-in') flowZoom.zoomCenter(1.25);
    if (button.dataset.action === 'zoom-out') flowZoom.zoomCenter(0.8);
  }));

  document.querySelectorAll('#view-tree [data-action]').forEach((button) => button.addEventListener('click', () => {
    if (button.dataset.action === 'fit') treeZoom.fit(40, 1);
    if (button.dataset.action === 'expand') { setTreeCollapsed(treeRoot, null); renderTree(false); }
    if (button.dataset.action === 'collapse') { setTreeCollapsed(treeRoot, 3); renderTree(false); }
  }));

  document.querySelectorAll('#view-graph [data-action]').forEach((button) => button.addEventListener('click', () => {
    const action = button.dataset.action;
    if (action === 'overview') { state.graphMode = 'overview'; renderGraph(); }
    if (action === 'fit') graphZoom.fit(40, 1);
    if (action === 'bad-only') {
      state.badOnly = !state.badOnly;
      button.classList.toggle('on', state.badOnly);
      if (state.graphMode === 'overview') renderOverview();
    }
  }));

  const searchInput = document.getElementById('graph-search');
  const searchResults = document.getElementById('graph-search-results');
  searchInput.addEventListener('input', () => {
    const text = searchInput.value.trim().toLowerCase();
    if (!text) { searchResults.style.display = 'none'; return; }
    const matches = data.units.filter((unit) => unit.id.toLowerCase().includes(text)
      || (unit.summary || '').toLowerCase().includes(text)).slice(0, 30);
    searchResults.innerHTML = matches.map((unit) => `<div data-id="${escapeHtml(unit.id)}">${badge(LAYER_NAME[unit.layer], LAYER_COLOR[unit.layer])} <span class="mono">${escapeHtml(unit.id)}</span></div>`).join('')
      || '<div class="muted">无匹配</div>';
    searchResults.style.display = 'block';
    searchResults.querySelectorAll('[data-id]').forEach((row) => row.addEventListener('click', () => {
      searchResults.style.display = 'none';
      searchInput.value = '';
      focusUnit(row.dataset.id);
    }));
  });
  searchInput.addEventListener('blur', () => setTimeout(() => { searchResults.style.display = 'none'; }, 200));

  const snapshotSelect = document.getElementById('snapshot');
  const snapshotNames = Object.keys(ALL);
  const SNAPSHOT_NAME = { before: '重构前', after: '重构后' };
  snapshotSelect.innerHTML = snapshotNames.map((name) => `<option value="${escapeHtml(name)}">${escapeHtml(SNAPSHOT_NAME[name] || name)}</option>`).join('');
  snapshotSelect.addEventListener('change', () => loadSnapshot(snapshotSelect.value));
  if (!snapshotNames.length) {
    document.body.innerHTML = '<p style="padding:40px">未找到数据文件 data/*.js。</p>';
    return;
  }
  loadSnapshot(snapshotNames[snapshotNames.length - 1]);
  snapshotSelect.value = snapshotNames[snapshotNames.length - 1];
})();
