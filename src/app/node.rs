use skia_safe::{ Canvas, Color4f, Paint, Rect };

#[derive(Debug)]
pub enum NodeBackground{
  Colour(( f32, f32, f32, f32 )),
  None
}

impl Default for NodeBackground{
  fn default() -> Self { Self::None }
}

#[derive(Default, Debug)]
pub struct Node{
  internal_text: Option<&'static str>,
  internal_children: Vec<Node>,
  internal_xy: Option<( f32, f32 )>,
  internal_wh: Option<( f32, f32 )>,
  internal_background: NodeBackground
}

impl Node{
  pub fn new() -> Node{
    Node {
      internal_text: None,
      internal_children: Vec::new(),
      internal_xy: None,
      internal_wh: None,
      internal_background: NodeBackground::None
    }
  }

  pub fn render( &self, canvas: &Canvas ){
    match self.internal_background{
      NodeBackground::None => {},
      NodeBackground::Colour(( r, g, b, a )) => {
        let paint = Paint::new(Color4f::new(r, g, b, a), None);
        let rect = Rect::from_xywh(
          self.internal_xy.unwrap().0,
          self.internal_xy.unwrap().1,
          self.internal_wh.unwrap().0,
          self.internal_wh.unwrap().1,
        );

        canvas.draw_rect(rect, &paint);
      }
    }
  }

  pub fn text( mut self, text: &'static str ) -> Node{
    self.internal_text = Some(text);
    self
  }

  pub fn child( mut self, node: Node ) -> Node{
    self.internal_children.push(node);
    self
  }

  pub fn xy( mut self, x: f32, y: f32 ) -> Node{
    self.internal_xy = Some(( x, y ));
    self
  }

  pub fn wh( mut self, w: f32, h: f32 ) -> Node{
    self.internal_wh = Some(( w, h ));
    self
  }

  pub fn background( mut self, bg: NodeBackground ) -> Node{
    self.internal_background = bg;
    self
  }
}