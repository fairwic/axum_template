//! Application-layer pagination DTO.

#[derive(Debug, Clone)]
pub struct PageResult<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}

impl<T> PageResult<T> {
    pub fn new(items: Vec<T>, total: i64, page: i64, page_size: i64) -> Self {
        let total_pages = (total as f64 / page_size as f64).ceil() as i64;
        Self {
            items,
            total,
            page,
            page_size,
            total_pages,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PageResult;

    #[test]
    fn computes_total_pages() {
        let page = PageResult::new(vec![1, 2, 3], 10, 1, 3);
        assert_eq!(page.total_pages, 4);
    }
}
