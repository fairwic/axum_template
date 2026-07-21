# DTO 规范实施记录

## 改造时间
2026-07-21

## 改造目标
加强 DTO 层规范，解决手动转换易错、缺乏验证、文档不完整的问题。

## 实施内容

### 1. 引入依赖
- **validator 0.18**：声明式验证框架（derive 宏 + 自定义验证函数）
- **lazy_static / regex**：辅助验证规则（可选，本次用自定义函数替代）

### 2. 重构 address_dto.rs（标准示例）

**验证规范**：
```rust
#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct CreateAddressRequest {
    #[validate(length(min = 1, max = 50, message = "..."))]
    pub name: String,
    
    #[validate(custom(function = "validate_phone", message = "..."))]
    pub phone: String,
}
```

**转换规范**：
```rust
impl From<CreateAddressRequest> for CreateAddressInput { ... }
impl From<Address> for AddressResponse { ... }
```

**文档规范**：
```rust
#[schema(example = json!({ "name": "张三", ... }))]
/// 创建收货地址请求
pub struct CreateAddressRequest {
    /// 收货人姓名（1-50 字符）
    #[schema(example = "张三")]
    pub name: String,
}
```

### 3. 重构 address_handler.rs

**验证调用**（Handler 入口）：
```rust
payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;
```

**简化转换**（移除手动逐字段构造）：
```rust
// Before: CreateAddressInput { name: payload.name, phone: payload.phone, ... }
// After:  payload.into()

let address = service.create(user_id, payload.into()).await?;
Ok(ApiResponse::success(address.into()))
```

### 4. 更新 README.md

新增"DTO 规范"子章节（在"开发规范" > "编码规范"下）：
- 分层与命名（Request/Response/Input）
- 验证规范（validator 使用模式）
- 转换规范（From trait 实现）
- 文档规范（OpenAPI 完整示例）
- 标准示例引用（`address_dto.rs`）

## 效果对比

| 指标 | 改造前 | 改造后 |
|------|--------|--------|
| **验证覆盖** | ❌ 无验证 | ✅ 所有字段带约束 |
| **转换方式** | 手动逐字段 | `From` trait 自动 |
| **文档完整性** | 部分字段缺 example | 全字段 + struct 级示例 |
| **Handler 代码行数** | ~20 行/接口 | ~10 行/接口 |
| **编译警告** | 0 | 0 |
| **测试通过** | 12 passed | 12 passed |

## 标准化收益

1. **类型安全**：编译期保证转换完整性（From trait），不会遗漏字段
2. **验证前置**：Handler 入口统一验证，业务层无需重复校验
3. **文档自动化**：OpenAPI 示例与代码同步，Swagger UI 直接可测试
4. **可维护性**：新增字段只需改 struct 定义 + From impl，Handler 无需改动

## 推广计划

- [x] address 模块（已完成，作为标准示例）
- [ ] snapshot 模块（`IngestProductRequest`/`IngestShopRequest`）
- [ ] 其他未来新增模块（遵循 `address_dto.rs` 模式）

## 参考文档

- README.md "开发规范" > "编码规范" > "DTO 规范"
- `crates/api/src/dtos/address_dto.rs`（标准实现）
- `crates/api/src/handlers/address_handler.rs`（验证与转换调用）
