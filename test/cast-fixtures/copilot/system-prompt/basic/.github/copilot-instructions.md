# Copilot Instructions - gastos-app

## Project Overview
**PHP MVC-style expense management application** with custom front controller, router, and role-based access control. Supports Admin, Manager, User, and Finance roles with complete gasto workflow: create → manager approval → reembolso management by finance.

**Tech Stack:**
- Backend: PHP 8+ with PDO (MySQL)
- Frontend: Bootstrap 5.3 (CDN), Select2 4.1.0, DataTables 1.11.5, Font Awesome 6.4.0, SweetAlert2
- Routing: Custom regex-based Router with {param} support
- Auth: Session-based with password_hash verification
- No external frameworks or composer dependencies

---

## Architecture & Core Flow

### Entry Point: `public/index.php`
1. Initializes sessions (`session_start()`)
2. Loads `app/autoload.php` (custom PSR-4 loader for `App\*` namespace)
3. Loads `app/Config/config.php` → reads `.env` into `$_ENV`
4. Enforces global auth via `AuthService::checkAuth()` (except `/login`, `/logout`)
5. Instantiates `Router` and dispatches routes from `routes/web.php`

### Router: `app/Core/Router.php`
- Matches routes using literal paths first, then regex patterns with `{param}` syntax
- Example: `/centros_costo/edit/{id}` extracts numeric ID and passes to handler
- Handlers are either:
  - **Closures**: Direct PHP callback functions
  - **Controller@method strings**: Auto-instantiates `App\Controllers\ControllerName` and calls method with extracted params

### Authentication & Authorization
- **AuthService** (`app/Services/AuthService.php`):
  - Manages session read/write
  - `user()` returns user profile with: `id, rut, nombre, email, departamento, role`
  - `logout()` clears session
  - `validateCredentials()` checks rut + password via `password_verify()`

- **AuthorizationService** (`app/Services/AuthorizationService.php`):
  - `isAdmin()`, `isManager()`, `isUser()`, `isFinance()` - role checks
  - `getManagedCentroCostos()` - returns array of centro IDs manager can modify
  - `canManageProyecto($proyecto_id)` - checks if manager can access specific proyecto
  - `getAccessibleGastos()` - returns gastos based on user role
  - `canReviewGasto($gastoId)` - checks if user can approve

### Database: `app/Core/Database.php`
- Static `connect()` returns PDO instance
- Uses prepared statements throughout
- Supports transactions: `beginTransaction()`, `commit()`, `rollBack()`
- All data returned as associative arrays

---

## Data Model & Tables

### Users Table
```sql
id, rut, nombre, email, password_hash, departamento, role (admin|manager|user|finance), 
banco, tipo_cuenta, numero_cuenta, titular_cuenta, rut_titular, estado (activo|inactivo)
```
**Test Users:**
- admin-001: role=admin, Depto=Administración
- manager-001 (Juan): role=manager, Depto=IT
- user-001 (Jorge): role=user, Depto=IT
- finance-001 (optional): role=finance, Depto=Finanzas

### Centros de Costo Table
```sql
id, nombre, descripcion, activo
```

### Proyectos Table
```sql
id, nombre, centro_costo_id, fecha_inicio, fecha_termino, presupuesto, activo
```
- `fecha_inicio`, `fecha_termino`, `presupuesto`: opcional, usado para reportes y alertas
- Presupuesto: triggers alerta cuando gastos tipo 'adelanto' lo superan

### Gastos Table
```sql
id, proyecto_id, fecha, monto, categoria_id, medio_pago_id, tipo (adelanto|reembolso|registro),
estado (pendiente|aprobado|rechazado|anulado|reembolsado),
created_by, reviewed_by, reviewed_at, review_comment,
reembolsado_by, reembolsado_at, activo
```
**Tipos de Gasto:**
- `adelanto`: Empresa entrega dinero previo (ej. viáticos). Descuenta presupuesto proyecto.
- `reembolso`: Usuario paga de su bolsillo, solicita reembolso. Requiere aprobación + gestión de finanzas.
- `registro`: Solo contable, sin reembolso. Requiere aprobación manager.

**Estado Workflow:**
- `pendiente` → Usuario crea gasto
- `aprobado` → Manager/encargado aprueba (o rechaza)
- `reembolsado` → Finance marca como reembolsado (solo para tipo reembolso)
- `rechazado` / `anulado` → Estados terminales

### Permission Tables
- **centro_costo_managers**: Links managers to centro_costo (manager can edit/create/manage)
- **proyecto_members**: Links users to proyectos with role (member|encargado); encargado can approve expenses
- Both have `activo` flag for soft-delete

---

## Controllers & Views

### Admin Routes
- **Dashboard**: `GET /dashboard` → DashboardController@index (role-specific cards)
- **Centros de Costo CRUD**: `/centros_costo` (list), `/centros_costo/create`, `/centros_costo/edit/{id}`, POST endpoints
- **Proyectos CRUD**: `/proyectos` (list), `/proyectos/create`, `/proyectos/edit/{id}`, POST endpoints
- **Manager Assignment**: `GET /admin/managers`, `POST /admin/managers/assign`, `POST /admin/managers/unassign/{id}`
- **Project Members**: 
  - `GET /admin/project-members` → list all projects + members
  - `POST /admin/project-members/assign` → multi-select assign (accepts `users_data[]`, `encargados[]`)
  - `POST /admin/project-members/unassign/{id}` → remove member
  - `POST /admin/project-members/promote/{id}` → upgrade member to encargado
  - `POST /admin/project-members/demote/{id}` → downgrade encargado to member

### Manager Routes
- **Project Members**: Same as admin but scoped to managed centros_costo
  - `/manager/project-members` (GET/POST with same actions)
  - Permission checks via `canManageProyecto()` in controller

### User Routes
- **Dashboard**: `GET /dashboard` (quick links, stats)
- **Profile**: `GET /profile` → ProfileController@index (save transfer data: banco, cuenta, etc.)
  - `POST /profile` → Update transfer data
- **Expenses**: `/gastos` (list), `/gastos/create`, `/gastos/edit/{id}` (own expenses only while pending)
- **Approvals**: `/approve/gastos` (if user is encargado in any proyecto)

### Manager Routes
- Same as user, plus scoped to managed centros_costo
- Can see gastos from managed centros, approve them

### Finance Routes
- **Reembolsos**: 
  - `GET /finance/reembolsos` (list reembolsos aprobados, filtrable por estado)
  - `POST /finance/reembolsos/mark/{id}` (mark gasto as reembolsado)

### Views Structure
```
app/Views/
├── Layout.php (master template with navbar)
├── auth/
│   ├── login.php
│   ├── dashboard.php (role-specific cards + stats)
│   ├── centros_costo/
│   │   ├── list.php (DataTable, AJAX toggle)
│   │   ├── create.php (card-based form)
│   │   └── edit.php (card-based form with toggle)
│   ├── proyectos/
│   │   ├── list.php (DataTable)
│   │   ├── create.php (form with centro selector)
│   │   └── edit.php (form with toggle)
│   ├── admin/
│   │   ├── project_members.php (2-step assign + member cards with promote/demote)
│   │   └── managers.php (assign managers to centros)
│   └── manager/
│       └── project_members.php (same UI as admin but scoped)
```

---

## Controllers Summary

### Core Application Flow
1. **AuthController** - Login/logout, session management
2. **DashboardController** - Role-specific dashboards (admin, manager, user, finance)
3. **CentroCostoController** - CRUD operations for cost centers (admin only)
4. **ProyectoController** - Project management with dates/budget/members (admin/manager)
5. **AdminProjectMembersController** - Multi-project member assignment (admin)
6. **AdminManagerController** - Manager assignment to cost centers (admin)
7. **ManagerProjectMembersController** - Project members scoped to managed cost centers
8. **GastoController** - Expense CRUD with types (adelanto|reembolso|registro), approval workflow
9. **ApprovalController** - Manager approval interface for pending expenses
10. **FinanceController** - Reembolso (reimbursement) management for finance role
11. **ProfileController** - User transfer data management for reembolsos

### Key Implementation Notes

**GastoController** (lines 1-400+)
- `create()`: Enforces `estado='pendiente'`, tipo validation, budget warnings for adelanto type
- `collectPayload()`: Validates and sanitizes input, prevents unauthorized estado changes
- `getAccessibleProyectosForForm()`: Uses prepared statements with proper parameter binding (PDO-safe)

**FinanceController** (lines 1-50+)
- `index()`: Lists gastos with `tipo='reembolso'`, estado filterable (aprobado|reembolsado|pending|rejected|annulled)
- `markReembolsado()`: Marks gasto as reembolsado, updates reembolsado_by/reembolsado_at timestamps

**ApprovalController** (lines 1-30+)
- `list()`: Shows pending gastos user can approve (as manager or proyecto encargado)
- `approve/reject()`: Updates estado and review metadata (reviewed_by, reviewed_at, review_comment)

---

## Frontend Stack & Styling

### CSS: `public/css/styles.css`
- Bootstrap 5.3 variables and overrides
- Select2 customization: multi-select badges, larger min-height for `.form-select-lg`, dropdown shadow
- DataTables styling: clean headers, responsive layout
- Utility classes: `.card-shadow`, `.alert-sm`, `.btn-group-sm`, `.dashboard-stat` (gradient backgrounds)
- Color scheme: Primary blue (#0d6efd), success (#198754), danger (#dc3545), warning (#ffc107), info (#0dcaf0)

### JavaScript: `public/js/app.js`
Auto-initialization functions (called on DOMContentLoaded):
- `initializeSweetAlert()` - SweetAlert2 defaults
- `initializeDataTables()` - `.data-table` class becomes sortable table
- `initializeSelect2()` - `.select2-single` and `.select2-multiple` become fancy selects
- `initializeBootstrapTooltips()` - Enable tooltips
- AJAX Helpers:
  - `toggleState(url, actionName)` - POST with SweetAlert confirmation, updates badge in-place
  - `showNotification(type, message)` - SweetAlert toast notification
  - `showConfirmation(title, message, onConfirm)` - Confirmation modal

### Layout.php Template
```php
- Navbar with role-specific menu items (admin/manager/user)
- Alert boxes for $_SESSION['success'/'error']
- Script loading order: Bootstrap JS → jQuery → DataTables → Select2 → app.js
- All views wrap with Layout::header() / Layout::footer()
```

---

## Key Implementation Patterns

### Multi-Select Member Assignment (Admin & Manager)
**Form Structure:**
```html
<select name="users_data[]" class="form-select-lg select2-multiple" multiple>
  <!-- All available users grouped by department -->
</select>
<select name="encargados[]" class="form-select-lg select2-multiple" multiple>
  <!-- Subset of users_data[] who should be encargados -->
</select>
```

**Controller Processing:**
```php
$usersData = array_map('intval', $_POST['users_data'] ?? []);
$encargados = array_map('intval', $_POST['encargados'] ?? []);

$db->beginTransaction();
foreach ($usersData as $userId) {
    $role = in_array($userId, $encargados) ? 'encargado' : 'member';
    // INSERT ... ON DUPLICATE KEY UPDATE
}
$db->commit();
```

**Member Card Display:**
- Shows user name, department, role badge
- Buttons: Promote/Demote (toggling role) + Remove (soft-delete)
- Role badge colors: Blue for member, Yellow for encargado

### AJAX Status Toggle (Centros/Proyectos)
- Toggle buttons call `toggleState('/path/resource/deactivate/{id}', 'action')` via JavaScript
- Controller returns JSON: `{success: true, message: "...", activo: true/false}`
- Frontend updates badge color + button state without page reload

### Dashboard Role-Specific Cards
- **Admin**: Quick links to Centros, Proyectos, Managers, Project Members + stats overview
- **Manager**: Quick links to Proyectos (managed), Members, Gastos approval stats
- **User**: Links to own Gastos, Projects assigned + pending approval items

---

## Config & Environment

### `.env` File (Required Keys)
```
DB_HOST=localhost
DB_PORT=3306
DB_NAME=gastos_app
DB_USER=root
DB_PASS=password
SESSION_NAME=gastos_session
```

### Testing Credentials
- **Admin**: RUT = admin-001, Password = (hashed in DB)
- **Manager**: RUT = manager-001, Password = (use database/schema.sql seed)
- **User**: RUT = user-001, Password = (use database/schema.sql seed)

---

## Conventions & Important Notes

1. **Custom Autoloader**: All classes must be in `App\*` namespace; composer autoload is NOT used
2. **Router Parameter Extraction**: Regex-based; `/edit/{id}` auto-extracts numeric IDs into method params
3. **Session-based Auth Only**: No JWT or middleware chains; check `$_SESSION['user']` directly or use AuthService/AuthorizationService
4. **Public Routes**: Only `/login` and `/logout` are public; all others enforce `AuthService::checkAuth()`
5. **PDO Prepared Statements**: Always use `:named` or `?` placeholders; never concatenate user input
6. **Soft Deletes**: Use `activo` flag (0/1) instead of actual deletion; queries always filter `WHERE activo = 1`
7. **CSS Centralized**: No inline styles in views; all styling in `public/css/styles.css`
8. **JavaScript Events**: Use `onclick=""` attributes in forms/buttons; app.js handles initialization

---

## Pending Tasks & Next Steps

### ✅ Completed (MVP)
- [x] **Admin Panel CRUD** - Centros, Proyectos, Managers, Project Members
- [x] **Dark Mode Theme** - Full theme system with CSS variables
- [x] **Select2 Multi-select** - Proper dropdown styling
- [x] **Gastos CRUD** - Create/list/edit views with compact design
- [x] **Expense Approval Workflow** - Manager/encargado approval with modal
- [x] **Finance Reembolso Management** - List, view transfer data, mark reembolsado
- [x] **User Profile** - Transfer data management for reembolsos
- [x] **Gasto Types** - Adelanto/Reembolso/Registro with proper workflow
- [x] **Proyecto Dates & Budget** - Optional fields, budget alert on adelanto
- [x] **UI Optimizations** - Compact icon-based tables, small fonts

### 📋 Next Priority (Polish & Testing)
- [ ] **Form Validation**: Enhanced client/server-side validation
- [ ] **Error Pages**: 404 and 500 error templates
- [ ] **Edge Cases**: Empty states, permission denials, validation errors
- [ ] **Testing**: Full workflow testing for all roles
- [ ] **Responsive Design**: Mobile/tablet layout verification

### 🎨 Future Enhancements (Post-MVP)
- [ ] **Advanced Filtering**: Centro, Proyecto, Date range, Status, User filters
- [ ] **Export**: CSV/PDF export for reports
- [ ] **Email Notifications**: Approval/rejection emails
- [ ] **Audit Log**: Change tracking
- [ ] **API**: JSON API for mobile apps

---

## Quick Reference: Common Tasks

### Add New Admin Route
```php
// routes/web.php
$router->get('/resource', 'ResourceController@index');
$router->post('/resource/create', 'ResourceController@create');

// app/Controllers/ResourceController.php
class ResourceController {
    public function index() {
        if (!AuthorizationService::isAdmin()) {
            http_response_code(403);
            return;
        }
        // ... fetch data, require view
    }
}

// app/Views/auth/resource/list.php
<?php use App\Views\Layout;
Layout::header();
// ... content with DataTable classes
Layout::footer();
?>
```

### Add New Select2 Multi-Select Field
```html
<select name="items[]" class="form-select form-select-lg select2-multiple" multiple>
    <option value="1">Option 1</option>
    <option value="2">Option 2</option>
</select>
```
No additional init needed; `app.js` auto-initializes on DOMContentLoaded.

### Use AJAX Toggle in View
```html
<button onclick="toggleState('/resource/toggle/<?= $id ?>', 'toggle')" class="btn btn-sm btn-outline-warning">
    <i class="fas fa-toggle-on"></i> Toggle
</button>
```

### Test User Login
1. Start dev server: `php -S localhost:8000 -t public/`
2. Navigate to `http://localhost:8000/login`
3. RUT: `admin-001`, Password: check database/schema.sql for test password
4. Should redirect to `/dashboard`

---

## File Structure Summary
```
gastos-app/
├── public/
│   ├── index.php (entry point)
│   ├── css/styles.css (all styling)
│   └── js/app.js (initialization + AJAX)
├── app/
│   ├── autoload.php (PSR-4 loader)
│   ├── Config/config.php (.env reader)
│   ├── Core/
│   │   ├── Router.php (regex routing)
│   │   ├── Database.php (PDO wrapper)
│   │   ├── Controller.php (base class)
│   │   └── AuthMiddleware.php (auth check)
│   ├── Services/
│   │   ├── AuthService.php (session mgmt)
│   │   └── AuthorizationService.php (permissions)
│   ├── Controllers/
│   │   ├── AuthController.php (login/logout)
│   │   ├── DashboardController.php
│   │   ├── CentroCostoController.php
│   │   ├── ProyectoController.php
│   │   ├── AdminProjectMembersController.php
│   │   ├── AdminManagerController.php
│   │   ├── ManagerProjectMembersController.php
│   │   ├── GastoController.php (create/list/edit/approve)
│   │   ├── ApprovalController.php (manager approval flow)
│   │   ├── FinanceController.php (reembolso management)
│   │   └── ProfileController.php (user transfer data)
│   ├── Models/
│   │   ├── CentroCosto.php (CRUD + soft-delete)
│   │   ├── Proyecto.php (CRUD with dates/budget)
│   │   ├── Gasto.php (CRUD with tipo field + budget tracking)
│   │   ├── Categoria.php (expense categories)
│   │   └── MedioPago.php (payment methods)
│   └── Views/
│       ├── Layout.php (master template)
│       └── auth/ (all authenticated views)
├── routes/
│   └── web.php (route definitions)
├── database/
│   └── schema.sql (MySQL schema + seed data)
├── .env.example (environment template)
└── .github/
    └── copilot-instructions.md (this file)
```

---

## Last Updated
- **2026-02-16 Final MVP**: ✅ Complete
  - Gasto workflow: user create → manager approval → finance reembolso
  - Finance role + reembolso management with transfer data
  - User profiles + transfer data for reembolsos
  - Project dates/budget tracking with alerts
  - Compact icon-based UI for gastos/approval lists
  - All 4 roles tested: admin, manager, user, finance
  - Test users: admin-001, manager-001, user-001 (+ finance-001 optional)
- **Status**: MVP Ready ✅ | Testing & Polish in progress
