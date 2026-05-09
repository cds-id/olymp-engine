# Blurp Engine - E-Commerce Platform Features

**Version:** 0.1.0  
**Stack:** Rust + Axum, PostgreSQL 16, Redis 7  
**Status:** MVP Development Phase

---

## Core Features

### 🔐 Authentication & Authorization
- **User Registration & Login** - Username/password authentication with JWT tokens
- **Magic Link Authentication** - Passwordless login via email (SHA256 hashed tokens)
- **Admin Role System** - Role-based access control for admin operations
- **Session Management** - Redis-backed session storage with configurable TTL
- **Rate Limiting** - Per-email and per-IP rate limiting for auth endpoints

### 📦 Product Catalog
- **Category Management** - Hierarchical product categories with soft delete
- **Product CRUD** - Full product lifecycle with publish/draft states
- **Product Variants** - SKU-based variants (size, color, etc.) with individual pricing
- **Stock Management** - Real-time inventory tracking per variant
- **Soft Delete** - Products/variants/categories marked deleted but preserved in DB
- **Media Management** - S3/R2 integration for product images (multiple per product/variant)
- **Visibility Control** - `is_active` flag for draft/published states, separate from deletion

### 🛒 Shopping Cart
- **Guest Cart** - Anonymous shopping with `X-Guest-ID` header
- **User Cart** - Persistent cart for authenticated users
- **Cart Wishlist** - Quick wishlist toggle within cart context
- **Stock Validation** - Real-time stock checks on add/update
- **Cart Expiry** - Automatic cleanup of abandoned carts

### ❤️ Wishlist
- **Dedicated Wishlist** - Separate wishlist system (guest + authenticated)
- **Wishlist Merge** - Merge guest wishlist into user account on login
- **Quick Check** - Check if variant is in wishlist
- **Bulk Operations** - Clear entire wishlist

### 📋 Order Management
- **Guest Checkout** - Order without account (email + guest tracking token)
- **User Checkout** - Authenticated user orders with order history
- **Shipping Quote** - Pre-checkout shipping cost calculation
- **Order Lookup** - Guest order tracking via email + token
- **Order History** - User order list with filtering
- **Stock Reservation** - Temporary stock hold during checkout (configurable TTL)
- **Order Status Tracking** - Pending → Processing → Shipped → Delivered → Cancelled

### 🚚 Shipping Integration
- **RajaOngkir API** - Indonesian shipping cost calculation
- **Location Search** - Province/city/district lookup with caching
- **Multi-Courier Support** - JNE, TIKI, POS Indonesia
- **Cost Calculation** - Real-time shipping quotes based on weight/destination
- **Redis Caching** - Location data cached for performance

### 💳 Payment Processing
- **Xendit Integration** - Indonesian payment gateway
- **Multiple Payment Methods** - E-wallet, bank transfer, credit card
- **Webhook Handling** - Automatic payment status updates
- **Payment Expiry** - Configurable payment window
- **Order-Payment Linking** - Automatic order status update on payment success

### 📧 Notifications
- **Email Service** - Mailgun (primary) + SMTP (fallback)
- **Transactional Emails** - Order confirmation, shipping updates, magic links
- **Template System** - Handlebars templates for email content
- **Background Jobs** - Async email sending via worker queue

### 👨‍💼 Admin Panel
- **Dashboard** - Revenue, orders, products overview
- **Revenue Reports** - GMV, order count, average order value
- **Product Reports** - Top products, low stock alerts
- **User Management** - List, view, update, delete users
- **Order Management** - View orders, update status, mark as shipped
- **Product Management** - Admin product/variant CRUD with stock updates
- **Settings** - Shipping and general configuration

### 🔧 Infrastructure
- **Background Workers** - Redis-backed job queue for async tasks
- **Stock Reservation Cleanup** - Automatic expiry of abandoned reservations
- **Database Migrations** - SQL migrations for schema versioning
- **Health Check** - `/health` endpoint for monitoring
- **Swagger UI** - API documentation (development only)
- **Environment-Based Config** - Dev/staging/production modes
- **Docker Compose** - Local development environment (Postgres + Redis)

---

## API Endpoints

### Authentication (`/api/auth`)
- `POST /register` - Create new user account
- `POST /login` - Username/password login
- `POST /logout` - Invalidate session
- `POST /magic-link` - Request passwordless login link
- `POST /callback` - Verify magic link token

### Catalog (`/api/catalog`)
- `GET /categories` - List categories
- `POST /categories` - Create category (admin)
- `GET /categories/{id}` - Get category details
- `PUT /categories/{id}` - Update category (admin)
- `DELETE /categories/{id}` - Soft delete category (admin)
- `GET /products` - List products (public: active only, admin: all)
- `POST /products` - Create product (admin)
- `GET /products/{id}` - Get product details
- `PUT /products/{id}` - Update product (admin)
- `DELETE /products/{id}` - Soft delete product (admin)
- `POST /products/{id}/variants` - Create variant (admin)
- `PUT /variants/{id}` - Update variant (admin)
- `DELETE /variants/{id}` - Soft delete variant (admin)
- `POST /products/{id}/media` - Upload product image (admin)
- `GET /products/{id}/media` - List product images
- `PUT /products/{id}/media/{media_id}` - Update image metadata (admin)
- `PUT /products/{id}/media/reorder` - Reorder images (admin)
- `POST /products/{id}/variants/{variant_id}/media` - Upload variant image (admin)
- `DELETE /media/{id}` - Delete image (admin)

### Cart (`/api/cart`)
- `GET /` - Get cart (guest or user)
- `DELETE /` - Clear cart
- `POST /items` - Add item to cart
- `PUT /items/{id}` - Update item quantity
- `DELETE /items/{id}` - Remove item from cart
- `GET /wishlist` - Get cart wishlist
- `POST /wishlist` - Toggle wishlist item

### Wishlist (`/api/wishlist`)
- `GET /` - Get wishlist
- `POST /` - Add to wishlist
- `DELETE /` - Remove from wishlist
- `DELETE /clear` - Clear entire wishlist
- `POST /merge` - Merge guest wishlist to user
- `GET /check/{variant_id}` - Check if variant in wishlist

### Orders (`/api/orders`)
- `POST /shipping-quote` - Calculate shipping cost
- `POST /checkout` - Create order (guest or user)
- `POST /guest/lookup` - Lookup guest order
- `GET /` - List user orders (authenticated)
- `GET /{id}` - Get order details

### Shipping (`/api/shipping`)
- `POST /cost` - Calculate shipping cost
- `GET /provinces` - List provinces
- `GET /cities` - List cities by province
- `GET /districts` - List districts by city

### Payments (`/api/payments`)
- `POST /` - Create payment
- `POST /webhook` - Xendit webhook handler
- `GET /{order_id}` - Get payment status

### Admin (`/api/admin`)
- `GET /dashboard` - Dashboard metrics
- `GET /reports/revenue` - Revenue report
- `GET /reports/products` - Product performance report
- `GET /users` - List users
- `GET /users/{id}` - Get user details
- `PUT /users/{id}` - Update user
- `DELETE /users/{id}` - Delete user
- `GET /orders` - List all orders
- `GET /orders/{id}` - Get order details
- `PUT /orders/{id}/status` - Update order status
- `PUT /orders/{id}/ship` - Mark order as shipped
- `GET /products` - List all products (including inactive)
- `GET /products/{id}` - Get product details
- `PUT /products/{id}` - Update product
- `DELETE /products/{id}` - Delete product
- `PUT /variants/{id}` - Update variant
- `PUT /variants/{id}/stock` - Update variant stock
- `GET /settings/shipping` - Get shipping settings
- `GET /settings/general` - Get general settings

### Documentation
- `GET /swagger-ui` - Swagger UI (development only)
- `GET /api-docs/openapi.json` - OpenAPI spec (development only)

---

## Technical Architecture

### Workspace Structure
```
blurp-engine/
├── crates/
│   ├── blurp-server/      # HTTP server, routing, main binary
│   ├── blurp-core/        # Shared types, errors, config
│   ├── blurp-auth/        # Authentication & authorization
│   ├── blurp-catalog/     # Product catalog management
│   ├── blurp-cart/        # Shopping cart operations
│   ├── blurp-wishlist/    # Wishlist management
│   ├── blurp-order/       # Order processing
│   ├── blurp-shipping/    # Shipping integration
│   ├── blurp-payment/     # Payment processing
│   ├── blurp-media/       # Media upload & storage
│   ├── blurp-admin/       # Admin operations
│   ├── blurp-notification/# Email notifications
│   └── blurp-worker/      # Background job processing
```

### Database Schema
- **users** - User accounts with admin flag
- **catalog.categories** - Product categories (soft delete)
- **catalog.products** - Products with `is_active` + `deleted_at`
- **catalog.variants** - Product variants (SKU, price, stock)
- **catalog.product_media** - Product/variant images
- **cart.carts** - Shopping carts (guest + user)
- **cart.cart_items** - Cart line items
- **cart.cart_wishlist** - Cart-based wishlist
- **wishlist.wishlists** - Dedicated wishlist (guest + user)
- **wishlist.wishlist_items** - Wishlist line items
- **orders.orders** - Orders (guest + user, nullable `user_id`)
- **orders.order_items** - Order line items
- **orders.fulfillment** - Shipping tracking info
- **orders.stock_reservations** - Temporary stock holds
- **payments.payments** - Payment records
- **payments.payment_events** - Payment status history

### Configuration
Environment variables (prefix `BLURP__`):
- `APP__NAME` - Application name
- `APP__URL` - Base URL
- `APP__ENVIRONMENT` - dev | staging | production
- `DATABASE__URL` - PostgreSQL connection string
- `DATABASE__MAX_CONNECTIONS` - Connection pool size
- `REDIS__URL` - Redis connection string
- `AUTH__JWT_SECRET` - JWT signing key
- `AUTH__COOKIE_SECRET` - Cookie encryption key
- `PAYMENT__PROVIDER` - xendit | midtrans
- `PAYMENT__API_KEY` - Payment gateway API key
- `SHIPPING__PROVIDER` - rajaongkir
- `SHIPPING__API_KEY` - Shipping API key
- `EMAIL__PROVIDER` - mailgun | ses | sendgrid
- `EMAIL__API_KEY` - Email service API key
- `STORAGE__ENDPOINT` - S3/R2 endpoint
- `STORAGE__BUCKET` - Storage bucket name
- `SERVER__BIND_ADDR` - Server bind address (default: 0.0.0.0:8080)

---

## Development Setup

### Prerequisites
- Rust 1.75+
- Docker & Docker Compose
- PostgreSQL 16
- Redis 7

### Quick Start
```bash
# Start infrastructure
docker compose up -d

# Copy environment config
cp .env.example .env.local
# Edit .env.local with your API keys

# Build and run
cargo build
./target/debug/blurp-server

# Server runs on http://localhost:8080
# Swagger UI: http://localhost:8080/swagger-ui (dev only)
```

### Database
```bash
# Connect to DB
docker exec -it blurp-postgres psql -U blurp -d blurp

# Run migrations (manual for now)
docker exec -i blurp-postgres psql -U blurp -d blurp < migrations/xxx.sql
```

### Testing
```bash
# Run all tests
cargo test

# Auth tests (single-threaded due to DB state)
cargo test -p blurp-auth -- --test-threads=1

# E2E tests (see docs/plans/2026-05-06-e2e-testing-handoff.md)
```

---

## Security Features

- **JWT Authentication** - Secure token-based auth with configurable TTL
- **Password Hashing** - Argon2 password hashing
- **Magic Link Security** - SHA256 hashed tokens, single-use, time-limited
- **Rate Limiting** - Prevent brute force attacks
- **Admin Middleware** - Role-based access control
- **SQL Injection Protection** - Parameterized queries via sqlx
- **CORS** - Configurable cross-origin policies
- **Environment-Based Security** - Swagger disabled in production
- **Guest ID Validation** - UUID format validation for guest operations

---

## Performance Optimizations

- **Redis Caching** - Location data, session storage
- **Connection Pooling** - PostgreSQL connection pool
- **Async/Await** - Non-blocking I/O throughout
- **Background Jobs** - Offload heavy tasks (email, cleanup)
- **Stock Reservation** - Prevent overselling with temporary holds
- **Soft Delete** - Fast logical deletes vs hard deletes
- **Index Optimization** - DB indexes on frequently queried fields

---

## Roadmap / Known Gaps

- [ ] OAuth/Google login integration
- [ ] Refresh token flow (currently access token only)
- [ ] Admin product creation endpoint (uses catalog endpoint)
- [ ] Migration runner in startup (currently manual)
- [ ] Cart/wishlist consolidation (two separate systems)
- [ ] Payment webhook verification stress testing
- [ ] Stock reservation cleanup stress testing
- [ ] Media upload endpoints (S3/R2 integration)
- [ ] Remaining E2E test coverage (18 endpoints untested)

---

## License

Proprietary - SoraStore Commerce Platform

---

**Last Updated:** 2026-05-06  
**Maintainer:** Blurp Engine Team
