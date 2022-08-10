
/// The struct to specify the series label of a target chart context
pub struct SeriesLabelStyle<'a, 'b, DB: DrawingBackend, CT: CoordTranslate> {
    target: &'b mut ChartContext<'a, DB, CT>,
    position: SeriesLabelPosition,
    legend_area_size: u32,
    border_style: ShapeStyle,
    background: ShapeStyle,
    label_font: Option<TextStyle<'b>>,
    margin: u32,
}

impl<'a, 'b, DB: DrawingBackend + 'a, CT: CoordTranslate> SeriesLabelStyle<'a, 'b, DB, CT> {
    pub(super) fn new(target: &'b mut ChartContext<'a, DB, CT>) -> Self {
        Self {
            target,
            position: SeriesLabelPosition::MiddleRight,
            legend_area_size: 30,
            border_style: (&TRANSPARENT).into(),
            background: (&TRANSPARENT).into(),
            label_font: None,
            margin: 10,
        }
    }

    /**
    Sets the series label positioning style

    `pos` - The positioning style

    See [`ChartContext::configure_series_labels()`] for more information and examples.
    */
    pub fn position(&mut self, pos: SeriesLabelPosition) -> &mut Self {
        self.position = pos;
        self
    }

    /**
    Sets the margin of the series label drawing area.

    - `value`: The size specification in backend units (pixels)

    See [`ChartContext::configure_series_labels()`] for more information and examples.
    */
    pub fn margin<S: SizeDesc>(&mut self, value: S) -> &mut Self {
        self.margin = value
            .in_pixels(&self.target.plotting_area().dim_in_pixel())
            .max(0) as u32;
        self
    }

    /**
    Sets the size of the legend area.

    `size` - The size of legend area in backend units (pixels)

    See [`ChartContext::configure_series_labels()`] for more information and examples.
    */
    pub fn legend_area_size<S: SizeDesc>(&mut self, size: S) -> &mut Self {
        let size = size
            .in_pixels(&self.target.plotting_area().dim_in_pixel())
            .max(0) as u32;
        self.legend_area_size = size;
        self
    }

    /**
    Sets the style of the label series area.

    `style` - The style of the border

    See [`ChartContext::configure_series_labels()`] for more information and examples.
    */
    pub fn border_style<S: Into<ShapeStyle>>(&mut self, style: S) -> &mut Self {
        self.border_style = style.into();
        self
    }

    /**
    Sets the background style of the label series area.

    `style` - The style of the border

    See [`ChartContext::configure_series_labels()`] for more information and examples.
    */
    pub fn background_style<S: Into<ShapeStyle>>(&mut self, style: S) -> &mut Self {
        self.background = style.into();
        self
    }

    /**
    Sets the font for series labels.

    `font` - Desired font

    See [`ChartContext::configure_series_labels()`] for more information and examples.
    */
    pub fn label_font<F: IntoTextStyle<'b>>(&mut self, font: F) -> &mut Self {
        self.label_font = Some(font.into_text_style(&self.target.plotting_area().dim_in_pixel()));
        self
    }

    /**
    Draws the series label area.

    See [`ChartContext::configure_series_labels()`] for more information and examples.
    */
    pub fn draw(&mut self) -> Result<(), DrawingAreaErrorKind<DB::ErrorType>> {
        let drawing_area = self.target.plotting_area().strip_coord_spec();

        // TODO: Issue #68 Currently generic font family doesn't load on OSX, change this after the issue
        // resolved
        let default_font = ("sans-serif", 12).into_font();
        let default_style: TextStyle = default_font.into();

        let font = {
            let mut temp = None;
            std::mem::swap(&mut self.label_font, &mut temp);
            temp.unwrap_or(default_style)
        };

        let mut label_element = MultiLineText::<_, &str>::new((0, 0), &font);
        let mut funcs = vec![];

        for anno in self.target.series_anno.iter() {
            let label_text = anno.get_label();
            let draw_func = anno.get_draw_func();

            if label_text.is_empty() && draw_func.is_none() {
                continue;
            }

            funcs.push(
                draw_func.unwrap_or(&|p: BackendCoord| EmptyElement::at(p).into_dyn()),
            );
            label_element.push_line(label_text);
        }

        let (mut w, mut h) = label_element.estimate_dimension().map_err(|e| {
            DrawingAreaErrorKind::BackendError(DrawingErrorKind::FontError(Box::new(e)))
        })?;

        let margin = self.margin as i32;

        w += self.legend_area_size as i32 + margin * 2;
        h += margin * 2;

        let (area_w, area_h) = drawing_area.dim_in_pixel();

        let (label_x, label_y) = self.position.layout_label_area((w, h), (area_w, area_h));

        label_element.relocate((
            label_x + self.legend_area_size as i32 + margin,
            label_y + margin,
        ));

        drawing_area.draw(&Rectangle::new(
            [(label_x, label_y), (label_x + w, label_y + h)],
            self.background.filled(),
        ))?;
        drawing_area.draw(&Rectangle::new(
            [(label_x, label_y), (label_x + w, label_y + h)],
            self.border_style,
        ))?;
        drawing_area.draw(&label_element)?;

        for (((_, y0), (_, y1)), make_elem) in label_element
            .compute_line_layout()
            .map_err(|e| {
                DrawingAreaErrorKind::BackendError(DrawingErrorKind::FontError(Box::new(e)))
            })?
            .into_iter()
            .zip(funcs.into_iter())
        {
            let legend_element = make_elem((label_x + margin, (y0 + y1) / 2));
            drawing_area.draw(&legend_element)?;
        }

        Ok(())
    }
}