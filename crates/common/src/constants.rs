//! 常量定义模块

/// 分页相关常量
pub mod pagination {
    /// 默认页码
    pub const DEFAULT_PAGE: i64 = 1;
    /// 默认每页数量
    pub const DEFAULT_PAGE_SIZE: i64 = 20;
    /// 最大每页数量
    pub const MAX_PAGE_SIZE: i64 = 100;
}

/// 缓存相关常量
pub mod cache {
    use std::time::Duration;

    /// 默认缓存过期时间
    pub const DEFAULT_TTL: Duration = Duration::from_secs(3600);
    /// 空值缓存过期时间
    pub const NULL_TTL: Duration = Duration::from_secs(60);
}
