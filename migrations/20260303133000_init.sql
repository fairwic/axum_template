-- core tables
CREATE TABLE IF NOT EXISTS stores (
    id VARCHAR(26) PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    address VARCHAR(255) NOT NULL,
    lat DOUBLE PRECISION NOT NULL,
    lng DOUBLE PRECISION NOT NULL,
    phone VARCHAR(20) NOT NULL,
    business_hours VARCHAR(50) NOT NULL,
    status VARCHAR(10) NOT NULL,
    delivery_radius_km DOUBLE PRECISION NOT NULL DEFAULT 3,
    delivery_fee_base INT NOT NULL DEFAULT 0,
    delivery_fee_per_km INT NOT NULL DEFAULT 0,
    runner_service_fee INT NOT NULL DEFAULT 2,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS admins (
    id VARCHAR(26) PRIMARY KEY,
    phone VARCHAR(20) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    role VARCHAR(20) NOT NULL,
    store_id VARCHAR(26),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT admins_store_id_fk FOREIGN KEY (store_id) REFERENCES stores (id)
);

CREATE INDEX IF NOT EXISTS admins_store_id_idx ON admins (store_id);

CREATE TABLE IF NOT EXISTS categories (
    id VARCHAR(26) PRIMARY KEY,
    store_id VARCHAR(26) NOT NULL,
    name VARCHAR(100) NOT NULL,
    sort_order INT NOT NULL DEFAULT 0,
    status VARCHAR(10) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT categories_store_id_fk FOREIGN KEY (store_id) REFERENCES stores (id)
);

CREATE INDEX IF NOT EXISTS categories_store_id_idx ON categories (store_id, sort_order);
CREATE UNIQUE INDEX IF NOT EXISTS categories_store_name_uniq ON categories (store_id, name);

CREATE TABLE IF NOT EXISTS products (
    id VARCHAR(26) PRIMARY KEY,
    store_id VARCHAR(26) NOT NULL,
    category_id VARCHAR(26) NOT NULL,
    title VARCHAR(200) NOT NULL,
    subtitle VARCHAR(200),
    cover_image TEXT NOT NULL,
    images JSONB NOT NULL DEFAULT '[]'::jsonb,
    price INT NOT NULL,
    original_price INT,
    stock INT NOT NULL DEFAULT 0,
    status VARCHAR(10) NOT NULL,
    tags JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT products_store_id_fk FOREIGN KEY (store_id) REFERENCES stores (id),
    CONSTRAINT products_category_id_fk FOREIGN KEY (category_id) REFERENCES categories (id)
);

CREATE INDEX IF NOT EXISTS products_store_category_idx ON products (store_id, category_id);
CREATE INDEX IF NOT EXISTS products_store_status_idx ON products (store_id, status);

CREATE TABLE IF NOT EXISTS carts (
    id VARCHAR(26) PRIMARY KEY,
    user_id VARCHAR(26) NOT NULL,
    store_id VARCHAR(26) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT carts_user_id_fk FOREIGN KEY (user_id) REFERENCES users (id),
    CONSTRAINT carts_store_id_fk FOREIGN KEY (store_id) REFERENCES stores (id),
    CONSTRAINT carts_user_store_uniq UNIQUE (user_id, store_id)
);

CREATE TABLE IF NOT EXISTS cart_items (
    cart_id VARCHAR(26) NOT NULL,
    product_id VARCHAR(26) NOT NULL,
    qty INT NOT NULL,
    price_snapshot INT NOT NULL,
    PRIMARY KEY (cart_id, product_id),
    CONSTRAINT cart_items_cart_id_fk FOREIGN KEY (cart_id) REFERENCES carts (id) ON DELETE CASCADE,
    CONSTRAINT cart_items_product_id_fk FOREIGN KEY (product_id) REFERENCES products (id)
);

CREATE INDEX IF NOT EXISTS cart_items_cart_id_idx ON cart_items (cart_id);
