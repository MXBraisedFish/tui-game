use crate::host_engine::services::{
  CanvasCell, ImageCellRect, ImageProtocol, ImageSignature, LayerImage, TextStyle,
};

pub type ImageId = u64;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ComposedCell {
  Empty,
  Text(CanvasCell),
  ImageAnchor(ImageId),
  ImageBody(ImageId),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComposedImage {
  pub id: ImageId,
  pub protocol: ImageProtocol,
  pub rect: ImageCellRect,
  pub signature: ImageSignature,
  pub sequence: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComposedFrame {
  width: u16,
  height: u16,
  cells: Vec<ComposedCell>,
  images: Vec<ComposedImage>,
  removed_regions: Vec<ImageCellRect>,
  image_dirty: bool,
}

impl ComposedFrame {
  pub fn new(width: u16, height: u16) -> Self {
    let len = width as usize * height as usize;
    Self {
      width,
      height,
      cells: vec![ComposedCell::Empty; len],
      images: Vec::new(),
      removed_regions: Vec::new(),
      image_dirty: false,
    }
  }

  pub fn width(&self) -> u16 {
    self.width
  }

  pub fn height(&self) -> u16 {
    self.height
  }

  pub fn images(&self) -> &[ComposedImage] {
    &self.images
  }

  pub fn removed_regions(&self) -> &[ImageCellRect] {
    &self.removed_regions
  }

  pub fn image_dirty(&self) -> bool {
    self.image_dirty
  }

  pub fn get(&self, x: u16, y: u16) -> Option<&ComposedCell> {
    let index = self.index(x, y)?;
    self.cells.get(index)
  }

  pub fn set(&mut self, x: u16, y: u16, cell: ComposedCell) {
    let Some(index) = self.index(x, y) else {
      return;
    };
    if let Some(target) = self.cells.get_mut(index) {
      *target = cell;
    }
  }

  pub fn add_image(&mut self, image: LayerImage) {
    self.images.push(ComposedImage {
      id: image.id,
      protocol: image.protocol,
      rect: image.rect,
      signature: image.signature,
      sequence: image.sequence,
    });
  }

  pub fn set_removed_regions(&mut self, regions: Vec<ImageCellRect>) {
    self.removed_regions = regions;
  }

  pub fn set_image_dirty(&mut self, dirty: bool) {
    self.image_dirty = dirty;
  }

  pub fn image_at(&self, id: ImageId) -> Option<&ComposedImage> {
    self.images.iter().find(|image| image.id == id)
  }

  pub fn blank_text_cell() -> CanvasCell {
    CanvasCell {
      ch: ' ',
      style: TextStyle::default(),
    }
  }

  fn index(&self, x: u16, y: u16) -> Option<usize> {
    if x >= self.width || y >= self.height {
      return None;
    }
    Some(y as usize * self.width as usize + x as usize)
  }
}
