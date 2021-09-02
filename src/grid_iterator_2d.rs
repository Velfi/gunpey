pub type GridXyIter<T> = Box<dyn Iterator<Item = (T, T)>>;

pub fn new_xy_iter(
    width: usize,
    height: usize,
    direction_x: GridIterDirectionX,
    direction_y: GridIterDirectionY,
) -> GridXyIter<usize> {
    use GridIterDirectionX::*;
    use GridIterDirectionY::*;
    match (direction_x, direction_y) {
        (LeftToRight, BottomToTop) => {
            Box::new((0..height).flat_map(move |y| (0..width).map(move |x| (x, y))))
        }
        (LeftToRight, TopToBottom) => Box::new(
            (0..height)
                .rev()
                .flat_map(move |y| (0..width).map(move |x| (x, y))),
        ),
        (RightToLeft, BottomToTop) => {
            Box::new((0..height).flat_map(move |y| (0..width).rev().map(move |x| (x, y))))
        }
        (RightToLeft, TopToBottom) => Box::new(
            (0..height)
                .rev()
                .flat_map(move |y| (0..width).rev().map(move |x| (x, y))),
        ),
    }
}
pub type GridIndexIter<T> = Box<dyn Iterator<Item = T>>;

pub fn new_index_iter(
    width: usize,
    height: usize,
    direction_x: GridIterDirectionX,
    direction_y: GridIterDirectionY,
) -> GridIndexIter<usize> {
    use GridIterDirectionX::*;
    use GridIterDirectionY::*;

    match (direction_x, direction_y) {
        (LeftToRight, BottomToTop) => {
            Box::new((0..height).flat_map(move |y| (0..width).map(move |x| x + y * width)))
        }
        (LeftToRight, TopToBottom) => Box::new(
            (0..height)
                .rev()
                .flat_map(move |y| (0..width).map(move |x| x + y * width)),
        ),
        (RightToLeft, BottomToTop) => {
            Box::new((0..height).flat_map(move |y| (0..width).rev().map(move |x| x + y * width)))
        }
        (RightToLeft, TopToBottom) => Box::new(
            (0..height)
                .rev()
                .flat_map(move |y| (0..width).rev().map(move |x| x + y * width)),
        ),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum GridIterDirectionX {
    LeftToRight,
    RightToLeft,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum GridIterDirectionY {
    BottomToTop,
    TopToBottom,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_left_to_right_top_to_bottom() {
        let width = 2;
        let height = 2;
        let direction_x = GridIterDirectionX::LeftToRight;
        let direction_y = GridIterDirectionY::TopToBottom;
        let iterator = &mut new_xy_iter(width, height, direction_x, direction_y);

        assert_eq!(iterator.next(), Some((0, 1)));
        assert_eq!(iterator.next(), Some((1, 1)));
        assert_eq!(iterator.next(), Some((0, 0)));
        assert_eq!(iterator.next(), Some((1, 0)));
        assert_eq!(iterator.next(), None);
    }

    #[test]
    fn test_left_to_right_bottom_to_top() {
        let width = 2;
        let height = 2;
        let direction_x = GridIterDirectionX::LeftToRight;
        let direction_y = GridIterDirectionY::BottomToTop;
        let iterator = &mut new_xy_iter(width, height, direction_x, direction_y);

        assert_eq!(iterator.next(), Some((0, 0)));
        assert_eq!(iterator.next(), Some((1, 0)));
        assert_eq!(iterator.next(), Some((0, 1)));
        assert_eq!(iterator.next(), Some((1, 1)));
        assert_eq!(iterator.next(), None);
    }

    #[test]
    fn test_right_to_left_top_to_bottom() {
        let width = 2;
        let height = 2;
        let direction_x = GridIterDirectionX::RightToLeft;
        let direction_y = GridIterDirectionY::TopToBottom;
        let iterator = &mut new_xy_iter(width, height, direction_x, direction_y);

        assert_eq!(iterator.next(), Some((1, 1)));
        assert_eq!(iterator.next(), Some((0, 1)));
        assert_eq!(iterator.next(), Some((1, 0)));
        assert_eq!(iterator.next(), Some((0, 0)));
        assert_eq!(iterator.next(), None);
    }

    #[test]
    fn test_right_to_left_bottom_to_top() {
        let width = 2;
        let height = 2;
        let direction_x = GridIterDirectionX::RightToLeft;
        let direction_y = GridIterDirectionY::BottomToTop;
        let iterator = &mut new_xy_iter(width, height, direction_x, direction_y);

        assert_eq!(iterator.next(), Some((1, 0)));
        assert_eq!(iterator.next(), Some((0, 0)));
        assert_eq!(iterator.next(), Some((1, 1)));
        assert_eq!(iterator.next(), Some((0, 1)));
        assert_eq!(iterator.next(), None);
    }
}
