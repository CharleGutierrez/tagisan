# LWC Anti-Patterns & Best Practices

Critical "Do's and Don'ts" for Lightning Web Component development.

## Template Expression Anti-Patterns

### ❌ NEVER: Use Ternary Operators in HTML Templates

**CRITICAL RULE**: LWC templates do NOT support ternary operators (`? :`), logical operators (`&&`, `||`), or complex JavaScript expressions directly in the HTML.

**WRONG - Ternary Operators:**
```html
<!-- ❌ WRONG - Ternary not supported -->
<div>{isActive ? 'Active' : 'Inactive'}</div>
<div>{count > 0 ? count : 'None'}</div>
<lightning-input label={isRequired ? 'Required' : 'Optional'}></lightning-input>
<div class={status === 'complete' ? 'slds-text-color_success' : 'slds-text-color_error'}></div>
<lightning-button disabled={!isValid ? true : false}></lightning-button>
```

**CORRECT - Use Getters:**
```javascript
// JavaScript
export default class MyComponent extends LightningElement {
    isActive = true;
    count = 5;
    isRequired = true;
    status = 'complete';
    isValid = false;
    
    get statusText() {
        return this.isActive ? 'Active' : 'Inactive';
    }
    
    get countDisplay() {
        return this.count > 0 ? this.count : 'None';
    }
    
    get fieldLabel() {
        return this.isRequired ? 'Required' : 'Optional';
    }
    
    get statusClass() {
        return this.status === 'complete' 
            ? 'slds-text-color_success' 
            : 'slds-text-color_error';
    }
    
    get isButtonDisabled() {
        return !this.isValid;
    }
}
```

```html
<!-- ✅ CORRECT HTML -->
<div>{statusText}</div>
<div>{countDisplay}</div>
<lightning-input label={fieldLabel}></lightning-input>
<div class={statusClass}></div>
<lightning-button disabled={isButtonDisabled}></lightning-button>
```

**WRONG - Logical Operators:**
```html
<!-- ❌ WRONG - Logical operators not supported in templates -->
<div>{user && user.name}</div>
<div>{items || 'No items'}</div>
<div>{user?.address?.city}</div>
<lightning-button disabled={!isValid && !isSaving}></lightning-button>
```

**CORRECT - Use Getters or if:true/if:false:**
```javascript
// JavaScript
get userName() {
    return this.user ? this.user.name : '';
}

get itemsDisplay() {
    return this.items || 'No items';
}

get userCity() {
    return this.user?.address?.city || '';
}

get shouldDisableButton() {
    return !this.isValid && !this.isSaving;
}
```

```html
<!-- ✅ CORRECT HTML -->
<div>{userName}</div>
<div>{itemsDisplay}</div>
<div>{userCity}</div>
<lightning-button disabled={shouldDisableButton}></lightning-button>

<!-- OR use conditional rendering -->
<template if:true={user}>
    <div>{user.name}</div>
</template>
```

**WRONG - Complex Expressions:**
```html
<!-- ❌ WRONG - Complex expressions not supported -->
<div>{items.length > 0 ? items.length + ' items' : 'No items'}</div>
<div>{amount * 1.1}</div>
<div>{firstName + ' ' + lastName}</div>
<div class={'slds-col slds-size_' + columnSize}></div>
```

**CORRECT - Use Getters:**
```javascript
// JavaScript
get itemCountText() {
    return this.items.length > 0 
        ? `${this.items.length} items` 
        : 'No items';
}

get amountWithTax() {
    return this.amount * 1.1;
}

get fullName() {
    return `${this.firstName} ${this.lastName}`;
}

get columnClass() {
    return `slds-col slds-size_${this.columnSize}`;
}
```

```html
<!-- ✅ CORRECT HTML -->
<div>{itemCountText}</div>
<div>{amountWithTax}</div>
<div>{fullName}</div>
<div class={columnClass}></div>
```

### Why This Matters

LWC uses a **restricted subset of JavaScript** in templates for:
- **Performance**: Expressions are optimized at compile time
- **Security**: Prevents code injection
- **Maintainability**: Keeps logic in JavaScript, not templates

**Supported in Templates:**
- Property access: `{propertyName}`
- Simple dot notation: `{object.property}`
- Optional chaining: `{object?.property}` (reads but doesn't write)
- Array/Object access: `{items[0]}`, `{record['Name']}`

**NOT Supported in Templates:**
- Ternary operators: `? :`
- Logical operators: `&&`, `||`, `!` (except in if:true/if:false)
- Arithmetic: `+`, `-`, `*`, `/`
- Comparison: `>`, `<`, `===`, `!==`
- Function calls: `formatDate(date)`
- Template literals: `` `${value}` ``

### Use Conditional Rendering Instead

**WRONG:**
```html
<div>{showMessage ? message : ''}</div>
<div>{hasError ? errorMessage : successMessage}</div>
```

**CORRECT:**
```html
<!-- Use if:true for conditional display -->
<template if:true={showMessage}>
    <div>{message}</div>
</template>

<!-- Use if:true/if:false for either/or -->
<template if:true={hasError}>
    <div>{errorMessage}</div>
</template>
<template if:false={hasError}>
    <div>{successMessage}</div>
</template>
```

## Component Selection Anti-Patterns

### ❌ NEVER: Use Generic HTML Form Elements

**WRONG:**
```html
<input type="text" placeholder="Name" />
<input type="email" />
<input type="checkbox" />
<select>
    <option>Option 1</option>
</select>
<button>Click me</button>
<textarea></textarea>
```

**CORRECT:**
```html
<lightning-input label="Name"></lightning-input>
<lightning-input type="email" label="Email"></lightning-input>
<lightning-input type="checkbox" label="Accept Terms"></lightning-input>
<lightning-combobox label="Select" options={options}></lightning-combobox>
<lightning-button label="Click me"></lightning-button>
<lightning-textarea label="Description"></lightning-textarea>
```

**Why:** Generic HTML elements:
- Don't integrate with Salesforce validation
- Don't follow SLDS styling automatically
- Don't provide accessibility features
- Don't work properly with Lightning Data Service
- Don't respect field-level security

### ❌ NEVER: Create Custom Dropdowns/Selects

**WRONG:**
```html
<div class="custom-dropdown">
    <div class="dropdown-toggle">Select...</div>
    <ul class="dropdown-menu">
        <li>Option 1</li>
        <li>Option 2</li>
    </ul>
</div>
```

**CORRECT:**
```html
<lightning-combobox
    label="Select Option"
    value={selectedValue}
    options={options}>
</lightning-combobox>
```

### ❌ NEVER: Build Custom Tables

**WRONG:**
```html
<table>
    <thead>
        <tr><th>Name</th><th>Amount</th></tr>
    </thead>
    <tbody>
        <tr for:each={items} for:item="item" key={item.id}>
            <td>{item.name}</td>
            <td>{item.amount}</td>
        </tr>
    </tbody>
</table>
```

**CORRECT:**
```html
<lightning-datatable
    key-field="id"
    data={items}
    columns={columns}>
</lightning-datatable>
```

### ❌ NEVER: Custom Modal Implementations (When Native Works)

**WRONG:**
```html
<div class="custom-modal" if:true={showModal}>
    <div class="modal-backdrop"></div>
    <div class="modal-content">
        <!-- Custom modal structure -->
    </div>
</div>
```

**BETTER (if needed):**
```html
<!-- Use SLDS modal markup for consistency -->
<section class="slds-modal slds-fade-in-open" if:true={showModal}>
    <div class="slds-modal__container">
        <!-- SLDS modal structure -->
    </div>
</section>
<div class="slds-backdrop slds-backdrop_open" if:true={showModal}></div>
```

**BEST:**
```html
<!-- Use lightning-modal when available (API v58+) -->
<lightning-modal>
    <!-- Content -->
</lightning-modal>
```

## Data Handling Anti-Patterns

### ❌ NEVER: Bypass Lightning Data Service When Available

**WRONG:**
```javascript
// Manual Apex call for standard CRUD
import getRecord from '@salesforce/apex/MyController.getRecord';

connectedCallback() {
    getRecord({ recordId: this.recordId })
        .then(result => this.record = result);
}
```

**CORRECT:**
```javascript
// Use LDS wire adapter
import { getRecord } from 'lightning/uiRecordApi';

@wire(getRecord, { recordId: '$recordId', fields: FIELDS })
record;
```

**Why:** LDS provides:
- Automatic caching
- Automatic refresh when data changes
- Respect for field-level security
- Better performance

### ❌ NEVER: Mutate @track or @api Properties Directly

**WRONG:**
```javascript
@track items = [];

addItem(newItem) {
    this.items.push(newItem); // Direct mutation - won't trigger reactivity
}
```

**CORRECT:**
```javascript
@track items = [];

addItem(newItem) {
    this.items = [...this.items, newItem]; // Create new array
}
```

### ❌ NEVER: Use DOM Queries for Component Communication

**WRONG:**
```javascript
// Parent component
this.template.querySelector('c-child').someProperty = 'value';
```

**CORRECT:**
```html
<!-- Use property binding -->
<c-child some-property={value}></c-child>
```

## Validation Anti-Patterns

### ❌ NEVER: Manual Field Validation When Built-in Exists

**WRONG:**
```javascript
handleSubmit() {
    const email = this.template.querySelector('[data-id="email"]').value;
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    
    if (!emailRegex.test(email)) {
        alert('Invalid email');
        return;
    }
    // ... more manual validation
}
```

**CORRECT:**
```html
<lightning-input
    type="email"
    label="Email"
    value={email}
    required
    message-when-value-missing="Email is required"
    message-when-type-mismatch="Please enter a valid email">
</lightning-input>
```

```javascript
handleSubmit() {
    const isValid = [...this.template.querySelectorAll('lightning-input')]
        .reduce((valid, input) => {
            input.reportValidity();
            return valid && input.checkValidity();
        }, true);
    
    if (isValid) {
        // Submit
    }
}
```

## Styling Anti-Patterns

### ❌ NEVER: Use Extensive Custom CSS Over SLDS

**WRONG:**
```css
.my-card {
    padding: 16px;
    margin: 8px;
    background: white;
    border: 1px solid #ccc;
    border-radius: 4px;
}

.my-button {
    background: #0176d3;
    color: white;
    padding: 8px 16px;
    border: none;
    border-radius: 4px;
}
```

**CORRECT:**
```html
<!-- Use SLDS utility classes -->
<div class="slds-p-around_medium slds-m-around_small slds-box">
    <lightning-button variant="brand" label="Action"></lightning-button>
</div>
```

### ❌ NEVER: Hardcode Colors or Spacing

**WRONG:**
```css
.my-element {
    color: #0176d3;
    margin-top: 16px;
}
```

**CORRECT:**
```css
.my-element {
    color: var(--slds-g-color-brand-base-1, #0176d3);
    margin-top: var(--lwc-spacingMedium, 1rem);
}
```

Or better, use SLDS classes:
```html
<div class="slds-text-color_brand slds-m-top_medium">
```

## Performance Anti-Patterns

### ❌ NEVER: Execute Logic in renderedCallback Without Guards

**WRONG:**
```javascript
renderedCallback() {
    // This runs after EVERY render - could be hundreds of times!
    this.expensiveOperation();
}
```

**CORRECT:**
```javascript
isRendered = false;

renderedCallback() {
    if (!this.isRendered) {
        this.isRendered = true;
        this.expensiveOperation();
    }
}
```

### ❌ NEVER: Make Apex Calls in Loops

**WRONG:**
```javascript
items.forEach(item => {
    updateRecord({ recordId: item.id }) // Makes N calls!
        .then(result => console.log(result));
});
```

**CORRECT:**
```javascript
// Bulk operation
updateRecords({ recordIds: items.map(i => i.id) })
    .then(result => console.log(result));
```

### ❌ NEVER: Load All Data Upfront

**WRONG:**
```javascript
@wire(getAllRecords) // Could be thousands of records
allRecords;
```

**CORRECT:**
```javascript
// Implement pagination
@wire(getRecords, { limit: 50, offset: '$pageOffset' })
pagedRecords;
```

## Security Anti-Patterns

### ❌ NEVER: Disable Security Checks

**WRONG:**
```apex
// Apex class
public without sharing class UnsafeController {
    @AuraEnabled
    public static List<Account> getAccounts() {
        return [SELECT Id, Name FROM Account]; // No CRUD/FLS checks!
    }
}
```

**CORRECT:**
```apex
public with sharing class SafeController {
    @AuraEnabled
    public static List<Account> getAccounts() {
        if (!Schema.sObjectType.Account.isAccessible()) {
            throw new AuraHandledException('No access to Account');
        }
        return [SELECT Id, Name FROM Account];
    }
}
```

### ❌ NEVER: Expose Sensitive Data in Components

**WRONG:**
```javascript
// Embedding API keys or secrets
const API_KEY = 'sk_live_abc123xyz'; // NEVER DO THIS!
```

**CORRECT:**
```apex
// Store in Custom Metadata or Named Credentials
// Access via secure Apex method
```

## Navigation Anti-Patterns

### ❌ NEVER: Use window.location for Internal Navigation

**WRONG:**
```javascript
navigateToRecord() {
    window.location.href = `/lightning/r/Account/${this.recordId}/view`;
}
```

**CORRECT:**
```javascript
import { NavigationMixin } from 'lightning/navigation';

export default class MyComp extends NavigationMixin(LightningElement) {
    navigateToRecord() {
        this[NavigationMixin.Navigate]({
            type: 'standard__recordPage',
            attributes: {
                recordId: this.recordId,
                actionName: 'view'
            }
        });
    }
}
```

## Component Architecture Anti-Patterns

### ❌ NEVER: Create Monolithic Components

**WRONG:**
```javascript
// One component doing everything
export default class MegaComponent extends LightningElement {
    // 500+ lines handling data, UI, validation, navigation, etc.
}
```

**CORRECT:**
```javascript
// Break into smaller, focused components
export default class ParentComponent extends LightningElement {
    // Orchestrates child components
}

// c-data-table (handles table display)
// c-filter-panel (handles filtering)
// c-action-buttons (handles actions)
```

### ❌ NEVER: Tightly Couple Components

**WRONG:**
```javascript
// Child component knows too much about parent
this.dispatchEvent(new CustomEvent('update', {
    detail: { 
        parentProperty: 'value',
        shouldRefreshParentData: true,
        parentCallbackToInvoke: 'handleUpdate'
    }
}));
```

**CORRECT:**
```javascript
// Child emits simple, generic events
this.dispatchEvent(new CustomEvent('itemselected', {
    detail: { itemId: this.selectedItemId }
}));
```

## Testing Anti-Patterns

### ❌ NEVER: Skip Unit Tests

**WRONG:**
```javascript
// No test file at all
```

**CORRECT:**
```javascript
// componentName.test.js
import { createElement } from 'lwc';
import ComponentName from 'c/componentName';

describe('c-component-name', () => {
    afterEach(() => {
        while (document.body.firstChild) {
            document.body.removeChild(document.body.firstChild);
        }
    });
    
    it('renders correctly', () => {
        const element = createElement('c-component-name', {
            is: ComponentName
        });
        document.body.appendChild(element);
        
        expect(element).toBeTruthy();
    });
});
```

## Common Mistakes Checklist

Before deploying your LWC, verify:

- [ ] **No ternary operators in HTML templates** (`? :` not allowed)
- [ ] **No logical operators in templates** (`&&`, `||` not allowed)
- [ ] **No complex expressions in templates** (use getters instead)
- [ ] **No generic HTML input elements** (`<input>`, `<select>`, `<button>`, `<textarea>`)
- [ ] **Use `lightning-*` components** for all UI elements
- [ ] **Use LDS** for standard CRUD operations
- [ ] **Use SLDS classes** for styling (not custom CSS)
- [ ] **Property binding** (not DOM manipulation) for parent-child communication
- [ ] **Built-in validation** (not custom validators)
- [ ] **Proper error handling** for all async operations
- [ ] **Navigation API** (not window.location)
- [ ] **Security** (with sharing, field-level security)
- [ ] **Unit tests** for all components
- [ ] **Accessibility** (labels, ARIA attributes)
- [ ] **No hardcoded strings** (use Custom Labels for text)
- [ ] **Proper loading states** (spinners for async operations)
- [ ] **Null/undefined checks** before accessing properties
- [ ] **Bulk operations** (not individual calls in loops)

## Decision Matrix: When to Use What

| Scenario | Don't Use | Use Instead |
|----------|-----------|-------------|
| Text input | `<input>` | `<lightning-input>` |
| Dropdown | `<select>` | `<lightning-combobox>` |
| Button | `<button>` | `<lightning-button>` |
| Table | `<table>` | `<lightning-datatable>` |
| Loading | Custom spinner | `<lightning-spinner>` |
| Date input | `<input type="date">` | `<lightning-input type="date">` |
| Checkbox | `<input type="checkbox">` | `<lightning-input type="checkbox">` |
| Get record | Custom Apex | `lightning/uiRecordApi` |
| Save record | Custom Apex | `lightning-record-form` or `lightning/uiRecordApi` |
| Navigation | `window.location` | `NavigationMixin` |
| Styling | Custom CSS | SLDS utility classes |
| Icons | Font/SVG icons | `<lightning-icon>` |
| Modal | Custom overlay | SLDS modal markup |
| Form validation | Manual checks | `reportValidity()` / `checkValidity()` |
| Formatted values | Manual formatting | `<lightning-formatted-*>` |

## When Generic HTML IS Acceptable

There are rare cases where generic HTML is appropriate:

1. **Semantic markup** - `<article>`, `<section>`, `<header>`, `<footer>`, `<nav>`, `<aside>`
2. **Content display** - `<p>`, `<h1>-<h6>`, `<span>`, `<div>` (for layout only)
3. **Lists** - `<ul>`, `<ol>`, `<li>` (for simple content lists, not interactive lists)
4. **Links** - `<a>` (but prefer `lightning-button` with navigation for actions)
5. **Tables** - Only for simple, non-interactive data where `lightning-datatable` is overkill

**Rule of thumb:** If the element requires user interaction or data binding, there's probably a Lightning component for it!

## Red Flags in Code Reviews

Watch for these patterns that indicate problems:

```javascript
// 🚩 Using querySelector for data access
const value = this.template.querySelector('input').value;

// 🚩 Manual DOM manipulation
this.template.querySelector('div').innerHTML = 'text';

// 🚩 Inline styles
style="color: red; padding: 10px"

// 🚩 No error handling
methodCall().then(result => this.data = result);

// 🚩 Hardcoded IDs
const userId = '005xxxxxxxxxxxxxxx';

// 🚩 Missing accessibility
<input /> // No label!

// 🚩 Direct mutation
this.items.push(newItem);

// 🚩 N+1 queries
items.forEach(item => this.callApex(item));

// 🚩 No null checks
const name = record.Account.Name; // What if Account is null?
```

## Summary

**Golden Rules:**
1. **Lightning components ALWAYS over generic HTML for interactive elements**
2. **LDS ALWAYS over custom Apex for standard CRUD**
3. **SLDS classes ALWAYS over custom CSS**
4. **Property binding ALWAYS over DOM queries**
5. **Built-in validation ALWAYS over manual checks**

Following these principles will make your LWC code:
- More maintainable
- More secure
- More performant
- More accessible
- More Salesforce-native
