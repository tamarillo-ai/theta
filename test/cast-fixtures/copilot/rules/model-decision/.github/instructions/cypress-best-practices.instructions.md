---
description: Cypress best practices for reliable test automation
applyTo: "**/*.cy.js"
---

# Cypress Best Practices

## Element Selection Priority

1. **BEST** - Data attributes: `[data-test="username"]`, `[data-cy="login"]`
2. **GOOD** - ID attributes: `#username` (if stable)
3. **OKAY** - Semantic classes: `.inventory-item` (if stable)
4. **AVOID** - Generic selectors: `button`, `input[type="text"]`
5. **NEVER** - Text content: `cy.contains('Add to cart')`

## Test Independence

- Each test must be self-contained
- Use `beforeEach` for setup
- Clean state between tests
- Never depend on test execution order
- Clear cookies/storage in `beforeEach`

## Waiting Strategies

### ❌ NEVER DO
```javascript
cy.wait(3000);  // Arbitrary waits
cy.get('button').click({ force: true });  // Forcing clicks
```

### ✅ ALWAYS DO
```javascript
cy.get('[data-cy="loading"]').should('not.exist');
cy.get('button').should('be.visible').and('not.be.disabled').click();
```

## Test Data Management

- Use `@faker-js/faker` for dynamic data
- Store static data in `cypress/fixtures/`
- Load environment variables from `.env`
- NEVER hardcode credentials

## Error Handling

- Implement comprehensive error handling in actions
- Use descriptive error messages
- Include Cypress.log() for debugging
- Take screenshots on failure

## Assertions

- Make assertions in action classes, NOT page objects
- Use descriptive assertion messages
- Group related assertions
- Test behavior, not implementation

## Common Anti-Patterns

### ❌ Direct DOM Access in Tests
```javascript
// WRONG
cy.get('#username').type('user');
cy.get('#password').type('pass');
```

### ✅ Use Page Objects
```javascript
// CORRECT
const loginPage = new LoginPage();
loginPage.login('user', 'pass');
```

### ❌ Business Logic in Page Objects
```javascript
// WRONG - in page object
login(username, password) {
  this.typeUsername(username);
  this.typePassword(password);
  this.clickLoginButton();
  cy.url().should('include', '/inventory');
}
```

### ✅ Keep Page Objects Simple
```javascript
// CORRECT - in page object
typeUsername(username) {
  return cy.get('[data-test="username"]').clear().type(username);
}

// Business logic in test file or action class
login(username, password) {
  loginPage.typeUsername(username);
  loginPage.typePassword(password);
  loginPage.clickLoginButton();
}
```

## Environment Configuration

```javascript
// .env file (root)
CYPRESS_BASE_URL=https://www.saucedemo.com/v1
CYPRESS_TEST_USERNAME=standard_user
CYPRESS_TEST_PASSWORD=secret_sauce
CYPRESS_DEFAULT_TIMEOUT=10000

// Usage in tests
const baseUrl = Cypress.env('BASE_URL');
cy.visit(baseUrl);
```

## Logging Best Practices

```javascript
Cypress.log({
  name: 'login',
  message: `Logging in as ${username}`,
  consoleProps: () => ({
    username,
    // Never log passwords
    url: Cypress.config('baseUrl')
  })
});
```

## Test Organization

- Group related tests with `describe` blocks
- Use `context` blocks for logical groupings
- Use descriptive test names that explain business value
- Focus on user stories, not technical implementation
- Keep tests short and focused

## Quality Checklist

- ✅ No arbitrary `cy.wait()` calls
- ✅ No `force: true` in clicks
- ✅ Uses proper selectors (data attributes)
- ✅ Tests are independent
- ✅ No hardcoded test data
- ✅ Proper error handling
- ✅ Descriptive logging