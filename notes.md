# Notes: 小卖部小程序后端 V1.0

## Sources
### Repo structure
- crates/api: 路由/handler/openapi
- crates/application: service/dto
- crates/domain: entity/repo trait/领域规则
- crates/infrastructure: postgres/redis/config
- common: AppError/ApiResponse/PagedResponse
- migrations: 当前仅 users 表示例

### Conventions
- 统一 ApiResponse/AppError
- SQLx query!/query_as!，.sqlx/ 需生成
- cache-aside 约定

### Decisions (from user)
- Sprint 1：门店/分类/商品/搜索/购物车
- 微信登录，支付 mock
- 距离：腾讯位置服务 API
- 超距可加价配送（按公里，向上取整 1km）
- 后台：管理员登录（手机号+密码），平台/门店双角色
- 会员默认全员会员
- 自动接单：Redis + worker
- 搜索：Postgres ILIKE
- 库存：下单即锁定
- 购物车：数据库
- 优惠券：仅占位
- 跑腿：固定费用
- 取消：仅未支付取消

## Synthesized Findings
### Gaps/Conflicts
- PRD 取消规则与当前选择冲突：已按“仅未支付取消”执行
- PRD 超距策略冲突：已选“允许加价配送”

### Implementation Constraints
- 需要扩展 DDD 层：domain/entities/repo, application/services, infrastructure/postgres, api/routes/handlers, migrations
- 需新增微信登录与管理员登录基础模型
- 需引入腾讯位置服务调用配置（app key/secret）
