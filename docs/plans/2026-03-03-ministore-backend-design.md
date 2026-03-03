# 小卖部小程序后端 V1.0 设计文档

## 目标与范围
- 目标：完成 V1.0 可上线后端，先落地 Sprint 1（门店/类目/商品/搜索/购物车 + 门店切换规则），并预留订单/跑腿扩展结构。
- 管理后台为独立前端项目，但由本后端提供 `admin` API。

## 关键决策
- 用户身份：微信登录（code→openid）。
- 支付：V1 使用 mock，预留微信支付入口与回调结构。
- 距离计算：腾讯位置服务 API。
- 超距配送：允许加价配送；按公里加价，距离向上取整到 1km。
- 取消规则：仅未支付取消（与 PRD 原文有差异）。
- 后台鉴权：管理员登录（手机号+密码），平台/门店双角色。
- 会员：默认全员会员；优惠券仅占位。
- 购物车：数据库持久化；库存在下单时锁定（Sprint 2）。

## 架构与分层
- API 层：`crates/api` 路由/handler/校验/响应封装。
- 应用层：`crates/application` Service/DTO，负责业务编排与事务边界。
- 领域层：`crates/domain` Entity/Repo Trait/领域规则。
- 基础设施：`crates/infrastructure` Postgres/Redis/第三方服务（腾讯 LBS、微信）。

## API 分区
- C 端：`/api/v1/*`
- 管理端：`/api/admin/v1/*`

## 数据模型（Sprint 1）
- users：openid(唯一), nickname, avatar, phone, is_member, created_at, updated_at
- admins：phone(唯一), password_hash, role(PLATFORM/STORE), store_id(nullable), created_at, updated_at
- stores：name, address, lat, lng, phone, business_hours, status, delivery_radius_km, delivery_fee_base, delivery_fee_per_km, runner_service_fee
- categories：store_id, name, sort_order, status
- products：store_id, category_id, title, subtitle, cover_image, images(json), price, original_price, stock, status, tags(json)
- carts：user_id, store_id, created_at, updated_at
- cart_items：cart_id, product_id, qty, price_snapshot

## 核心流程
- 门店列表：调用腾讯 LBS 计算距离，按距离排序返回；同时返回可配送与配送费预估。
- 购物车：强制 `user_id + store_id` 绑定，切店前端确认后调用清空接口。
- 搜索：Postgres ILIKE（title/subtitle）。
- 管理端：门店/类目/商品 CRUD 与上下架/库存调整。

## 外部依赖
- 腾讯位置服务：距离计算与排序。
- 微信登录：code→openid。
- 支付 mock：后续替换微信支付。

## 错误与响应
- 统一使用 `ApiResponse` 与 `AppError`。
- 业务校验失败返回 HTTP 200 + error body。

## 测试策略
- 领域层：实体校验单元测试。
- 应用层：Service 业务流程测试（含仓储 mock）。
- API 层：关键路由集成测试（参考现有 user_routes_test 模式）。

## 安全与权限
- Admin 端 JWT 鉴权，角色区分平台/门店。
- C 端 JWT 鉴权，openid 绑定用户。

## 后续扩展（Sprint 2/3）
- 订单/支付/退款、跑腿订单与状态机、自动接单 worker。
