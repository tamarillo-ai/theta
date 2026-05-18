# AGENTS.md

This file provides guidance to Codex (Codex.ai/code) when working with code in this repository.

## Project Overview

Convision is a monorepo for an optics clinic management system with two sub-projects:
- **`convision-api/`** — Laravel 8 REST API (backend)
- **`convision-front/`** — React 18 + TypeScript SPA (frontend)

---

## Commands

### Backend (`convision-api/`)

```bash
# Install dependencies
composer install

# Initial setup (run once)
php artisan key:generate
php artisan jwt:secret
php artisan migrate --seed

# Si POST /api/v1/auth/login devuelve 401 con admin@convision.com / password:
# suele ser BD sin usuarios (migrate sin --seed). Desde convision-api:
php artisan convision:ensure-dev-users

# Development server (port 8000)
php artisan serve

# Run all tests
php artisan test

# Run a single test file
php artisan test tests/Feature/YourTest.php

# Refresh database with seeders
php artisan migrate:fresh --seed

# Docker — stack completo (MySQL + API + Frontend + phpMyAdmin)
# Desde la raíz del proyecto:
cp convision-api/.env.docker.example convision-api/.env.docker
docker compose -f docker/docker-compose.yml up -d

# Apple Silicon (M1/M2/M3):
DOCKER_PLATFORM=linux/arm64/v8 docker compose -f docker/docker-compose.yml up -d

# URLs:
#   Frontend:    http://localhost:4300
#   API:         http://localhost:8000
#   phpMyAdmin:  http://localhost:8080
#
# La primera vez instala deps, corre migraciones y seeds automáticamente.
```

### Frontend (`convision-front/`)

```bash
# Install dependencies
npm install

# Development server (port 4300 — ver `convision-front/vite.config.ts`)
npm run dev

# Production build
npm run build

# Lint
npm run lint
```

### Test Credentials

```bash
# Get a backend JWT token
curl --location 'http://localhost:8000/api/v1/auth/login' \
--header 'Content-Type: application/json' \
--data-raw '{"email":"admin@convision.com","password":"password"}'
```

Roles: `admin@convision.com`, `specialist@convision.com`, `receptionist@convision.com` — all use password `password`.

---

## Architecture

### Backend (Laravel)

**Layer structure:**
1. **Routes** → `routes/api.php` — all endpoints under `/api/v1/`
2. **Controllers** → `app/Http/Controllers/Api/V1/` — thin, delegates to services
3. **Form Requests** → `app/Http/Requests/Api/V1/{Entity}/{Action}{Entity}Request.php` — all validation lives here
4. **Services** → `app/Services/` — all business logic
5. **Resources** → `app/Http/Resources/V1/{Category}/` — all API responses
6. **Models** → `app/Models/` — include the `ApiFilterable` trait for consistent filtering

**Controller pattern (index/show/store/update/destroy):**

```php
// index — always use apiFilter and cap pagination at 100
public function index(Request $request)
{
    $perPage = min(max(1, (int)$request->get('per_page', 15)), 100);
    return new ModelCollection(Model::apiFilter($request)->paginate($perPage));
}

// store/update — use validated() from Form Request
public function store(StoreModelRequest $request)
{
    return new ModelResource(Model::create($request->validated()));
}

// destroy — return 204
public function destroy($id)
{
    Model::findOrFail($id)->delete();
    return response()->json(null, 204);
}
```

**Strict rules:**
- Never use `response()->json()` — always use API Resources
- Never validate in controllers — always use Form Request classes
- Every new model needs a migration, factory, and seeder
- Use eager loading to avoid N+1 queries

**ApiFilterable:** Add the `ApiFilterable` trait to any model that needs filterable list endpoints. Controllers call `Model::apiFilter($request)`. Frontend sends filter params as `s_f` (fields) and `s_v` (values) as JSON.

### Frontend (React)

**Structure:**
- `src/pages/` — role-based pages: `admin/`, `specialist/`, `receptionist/`
- `src/components/` — shared components; `ui/` contains shadcn-ui components
- `src/services/` — all API calls via axios (never call axios directly in components)
- `src/contexts/` — AuthContext for authentication state
- `App.tsx` — React Router v6 with role-based route protection

**Key conventions:**
- All frontend text must be in **Spanish**
- All tables must use the **EntityTable** / **DataTable** component — do not build custom table UIs
- All date pickers must use the **DatePicker** component
- Components must stay under **200 lines** — extract logic into custom hooks or sub-components
- Use Tailwind CSS for all styling — no inline styles or separate CSS files
- Forms use React Hook Form + Zod validation
- API calls go in `src/services/`, consumed via React Query hooks, never directly in components

**Discount flow for lenses:**
```ts
// 1. Check lens.has_discounts === true
// 2. Get best discount
const bestDiscount = await discountService.getBestDiscount(lensId, patientId?);
// 3. Calculate final price
const finalPrice = discountService.calculateDiscountedPrice(originalPrice, bestDiscount.discount_percentage);
```

**Vite proxy:** In development, `/api` requests are proxied to `http://localhost:8000`.

### Roles

| Role | Access |
|---|---|
| Admin | Full system |
| Specialist | Appointments, prescriptions, patient history |
| Receptionist | Patients, sales, quotes, appointments |
