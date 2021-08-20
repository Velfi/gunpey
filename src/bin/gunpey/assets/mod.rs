use druid::{
    piet::InterpolationMode,
    widget::{FillStrat, Image},
    ImageBuf,
};

// Creating functions to load assets is boring and boilerplate-ey, this macro makes it concise
macro_rules! image_asset {
    ( $asset_name:ident ) => {
        pub fn $asset_name() -> Image {
            let $asset_name =
                ImageBuf::from_data(include_bytes!(concat!(stringify!($asset_name), ".png")))
                    .unwrap();
            Image::new($asset_name)
                .fill_mode(FillStrat::Fill)
                .interpolation_mode(InterpolationMode::NearestNeighbor)
        }
    };
}

image_asset!(active_caret);
image_asset!(active_inverted_caret);
image_asset!(active_left_slash);
image_asset!(active_right_slash);
image_asset!(caret);
image_asset!(inverted_caret);
image_asset!(left_slash);
image_asset!(right_slash);
image_asset!(empty_cell);
// image_asset!(cursor);
