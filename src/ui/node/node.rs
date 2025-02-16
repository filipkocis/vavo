use crate::prelude::Color;

/// Defines the style properties of an Ui Entity in a similar fashion to CSS

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum Val {
    #[default]
    Auto,
    Px(f32),
    Rem(f32),
    Percent(f32),
    Vw(f32),
    Vh(f32),
}

#[derive(Default, Debug, Clone, Copy)]
pub struct UiRect {
    pub left: Val,
    pub right: Val,
    pub top: Val,
    pub bottom: Val,
}

impl UiRect {
    pub fn new(left: Val, right: Val, top: Val, bottom: Val) -> Self {
        Self {
            left,
            right,
            top,
            bottom,
        }
    }
    
    pub fn left(val: Val) -> Self {
        Self {
            left: val,
            ..Default::default()
        }
    }

    pub fn right(val: Val) -> Self {
        Self {
            right: val,
            ..Default::default()
        }
    }
    
    pub fn top(val: Val) -> Self {
        Self {
            top: val,
            ..Default::default()
        }
    }
    
    pub fn bottom(val: Val) -> Self {
        Self {
            bottom: val,
            ..Default::default()
        }
    }

    pub fn all(val: Val) -> Self {
        Self {
            left: val,
            right: val,
            top: val,
            bottom: val,
        }
    }

    pub fn vertical(v: Val) -> Self {
        Self {
            left: Val::Auto,
            right: Val::Auto,
            top: v,
            bottom: v,
        }
    }

    pub fn horizontal(h: Val) -> Self {
        Self {
            left: h,
            right: h,
            top: Val::Auto,
            bottom: Val::Auto,
        }
    }

    pub fn vh(v: Val, h: Val) -> Self {
        Self {
            left: h,
            right: h,
            top: v,
            bottom: v,
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum Display {
    Flex,
    Grid,
    #[default]
    Block,
    None,
}

#[derive(Default, Debug, Clone, Copy)]
pub enum Position {
    #[default]
    Relative,
    Absolute,
}

#[derive(Default, Debug, Clone, Copy)]
pub enum FlexDirection {
    #[default]
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}

impl FlexDirection {
    /// True for `RowReverse` and `ColumnReverse`
    pub fn is_reverse(&self) -> bool {
        match self {
            Self::RowReverse | Self::ColumnReverse => true,
            _ => false,
        }
    }

    /// True for `Row` and `RowReverse`
    pub fn is_row(&self) -> bool {
        match self {
            Self::Row | Self::RowReverse => true,
            _ => false,
        }
    }

    /// True for `Column` and `ColumnReverse`
    pub fn is_column(&self) -> bool {
        match self {
            Self::Column | Self::ColumnReverse => true,
            _ => false,
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub enum JustifyContent {
    #[default]
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum AlignItems {
    FlexStart,
    FlexEnd,
    Center,
    #[default]
    Stretch,
    // Baseline,
}

#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub enum BoxSizing {
    ContentBox,
    #[default]
    BorderBox,
}

#[derive(Default, Debug, Clone)]
#[derive(crate::macros::Component)]
pub struct Node {
    pub background_color: Color,
    /// None - inherit
    /// Some - override
    pub color: Option<Color>,
    pub border_color: Color,

    pub display: Display,
    pub position: Position,
    pub z_index: i32,
    pub box_sizing: BoxSizing,

    pub flex_direction: FlexDirection,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
    
    pub grid_template_columns: Vec<Val>,
    pub grid_template_rows: Vec<Val>,

    pub column_gap: Val,
    pub row_gap: Val,

    pub padding: UiRect,
    pub margin: UiRect,
    pub border: UiRect,

    pub width: Val,
    pub min_width: Val,
    pub max_width: Val,
    pub height: Val,
    pub min_height: Val,
    pub max_height: Val,
}

impl Node {
    /// True for Display::Flex and flex_direction.is_column()
    pub fn is_flex_column(&self) -> bool {
        self.display == Display::Flex && self.flex_direction.is_column()
    }

    /// True for Display::Flex and flex_direction.is_row()
    pub fn is_flex_row(&self) -> bool {
        self.display == Display::Flex && self.flex_direction.is_row()
    }

    /// True for AlignItems::Stretch and node.is_flex_column()
    pub fn does_stretch_width(&self) -> bool {
        self.is_flex_column() && self.align_items == AlignItems::Stretch
    }

    /// True for AlignItems::Stretch and node.is_flex_row()
    pub fn does_stretch_height(&self) -> bool {
        self.is_flex_row() && self.align_items == AlignItems::Stretch
    }
}
