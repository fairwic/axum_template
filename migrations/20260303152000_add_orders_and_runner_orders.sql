CREATE TABLE IF NOT EXISTS goods_orders (
    id VARCHAR(26) PRIMARY KEY,
    user_id VARCHAR(26) NOT NULL,
    store_id VARCHAR(26) NOT NULL,
    delivery_type VARCHAR(20) NOT NULL,
    status VARCHAR(20) NOT NULL,
    items JSONB NOT NULL,
    amount_goods INT NOT NULL,
    amount_delivery_fee INT NOT NULL,
    amount_discount INT NOT NULL DEFAULT 0,
    amount_payable INT NOT NULL,
    distance_km DOUBLE PRECISION,
    address_snapshot JSONB,
    store_snapshot JSONB,
    remark TEXT,
    pay_status VARCHAR(20) NOT NULL,
    pay_time TIMESTAMPTZ,
    cancel_reason TEXT,
    cancel_time TIMESTAMPTZ,
    accept_time TIMESTAMPTZ,
    complete_time TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT goods_orders_user_id_fk FOREIGN KEY (user_id) REFERENCES users (id),
    CONSTRAINT goods_orders_store_id_fk FOREIGN KEY (store_id) REFERENCES stores (id)
);

CREATE INDEX IF NOT EXISTS goods_orders_user_idx ON goods_orders (user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS goods_orders_store_idx ON goods_orders (store_id, created_at DESC);
CREATE INDEX IF NOT EXISTS goods_orders_status_idx ON goods_orders (status, created_at DESC);

CREATE TABLE IF NOT EXISTS runner_orders (
    id VARCHAR(26) PRIMARY KEY,
    user_id VARCHAR(26) NOT NULL,
    store_id VARCHAR(26) NOT NULL,
    status VARCHAR(20) NOT NULL,
    express_company VARCHAR(64) NOT NULL,
    pickup_code TEXT NOT NULL,
    delivery_address TEXT NOT NULL,
    receiver_name VARCHAR(64) NOT NULL,
    receiver_phone VARCHAR(20) NOT NULL,
    remark TEXT,
    service_fee INT NOT NULL,
    distance_km DOUBLE PRECISION,
    amount_payable INT NOT NULL,
    pay_status VARCHAR(20) NOT NULL,
    pay_time TIMESTAMPTZ,
    cancel_reason TEXT,
    cancel_time TIMESTAMPTZ,
    accept_time TIMESTAMPTZ,
    delivered_time TIMESTAMPTZ,
    complete_time TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT runner_orders_user_id_fk FOREIGN KEY (user_id) REFERENCES users (id),
    CONSTRAINT runner_orders_store_id_fk FOREIGN KEY (store_id) REFERENCES stores (id)
);

CREATE INDEX IF NOT EXISTS runner_orders_user_idx ON runner_orders (user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS runner_orders_store_idx ON runner_orders (store_id, created_at DESC);
CREATE INDEX IF NOT EXISTS runner_orders_status_idx ON runner_orders (status, created_at DESC);
