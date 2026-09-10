---
name: lwc-development
description: Expert guidance for Lightning Web Component (LWC) development on Salesforce Platform.
  Use this skill when working with Salesforce LWC projects, creating or modifying
  Lightning components, fixing LWC code, or when the user mentions Salesforce, Lightning
  Web Components, SLDS, lightning namespace components, or .js-meta.xml files. Always
  prefer standard Lightning components over generic HTML/web components.
...
---
# Lightning Web Components Development Guide

## Core Principle: Always Use Lightning Namespace Components

**CRITICAL**: When building Lightning Web Components for Salesforce, ALWAYS use `lightning-*` namespace components instead of generic HTML elements. This ensures proper Salesforce integration, styling, accessibility, and platform compatibility.

## Component Selection Guide

### User Input Components

| Use This | NOT This | When |
|----------|----------|------|
| `<lightning-input>` | `<input>` | Text fields, numbers, dates, checkboxes, toggles |
| `<lightning-textarea>` | `<textarea>` | Multi-line text input |
| `<lightning-combobox>` | `<select>` | Dropdown selections with search |
| `<lightning-dual-listbox>` | `<select multiple>` | Moving items between lists |
| `<lightning-input-address>` | Multiple `<input>` fields | Address entry with validation |
| `<lightning-input-location>` | Map/location inputs | Geographic location selection |
| `<lightning-slider>` | `<input type="range">` | Numeric range selection |
| `<lightning-radio-group>` | `<input type="radio">` | Radio button groups |
| `<lightning-checkbox-group>` | `<input type="checkbox">` | Checkbox groups |

### Buttons & Actions

| Use This | NOT This | When |
|----------|----------|------|
| `<lightning-button>` | `<button>` | Standard actions, form submissions |
| `<lightning-button-icon>` | `<button>` with icon | Icon-only buttons |
| `<lightning-button-menu>` | `<select>` or custom menu | Dropdown action menus |
| `<lightning-button-group>` | Multiple `<button>` | Related button groups |

### Layout & Structure

| Use This | NOT This | When |
|----------|----------|------|
| `<lightning-card>` | `<div>` with custom styling | Contained content sections |
| `<lightning-layout>` | `<div>` with flexbox | Responsive grid layouts |
| `<lightning-layout-item>` | `<div>` | Items within lightning-layout |
| `<lightning-accordion>` | Custom collapsible `<div>` | Expandable/collapsible sections |
| `<lightning-tab>` / `<lightning-tabset>` | Custom tab UI | Tabbed interfaces |
| `<lightning-carousel>` | Custom slideshow | Image carousels |
| `<lightning-tile>` | `<div>` | Compact record display |

### Data Display

| Use This | NOT This | When |
|----------|----------|------|
| `<lightning-datatable>` | `<table>` | Tabular data with sorting/filtering |
| `<lightning-tree>` | Nested `<ul>` | Hierarchical data |
| `<lightning-tree-grid>` | Complex nested tables | Tree structure with columns |
| `<lightning-formatted-*>` | Manual formatting | Display formatted values (date, time, currency, etc.) |
| `<lightning-output-field>` | `<span>` or `<div>` | Display Salesforce field values |
| `<lightning-avatar>` | `<img>` for user photos | User profile pictures |
| `<lightning-badge>` | `<span>` with styling | Status indicators |
| `<lightning-icon>` | SVG or font icons | SLDS icons |
| `<lightning-progress-bar>` | Custom progress indicators | Progress visualization |
| `<lightning-progress-indicator>` | Custom step indicators | Multi-step processes |

### Record Interaction

| Use This | NOT This | When |
|----------|----------|------|
| `<lightning-record-form>` | Custom form with fields | Quick record create/edit/view forms |
| `<lightning-record-edit-form>` | Manual field inputs | Custom record editing with validation |
| `<lightning-record-view-form>` | Manual field display | Display record fields |
| `<lightning-output-field>` | Manual field rendering | Display fields in view forms |

### Navigation & Feedback

| Use This | NOT This | When |
|----------|----------|------|
| `<lightning-spinner>` | Custom loading animation | Loading states |
| `<lightning-pill>` | `<span>` with close button | Removable tags/labels |
| `<lightning-helptext>` | Custom tooltip | Help text/tooltips |
| `<lightning-breadcrumb>` / `<lightning-breadcrumbs>` | Custom breadcrumb trail | Navigation hierarchy |

## Essential LWC Patterns

### 1. Component Structure

```javascript
// componentName.js
import { LightningElement, api, track, wire } from 'lwc';

export default class ComponentName extends LightningElement {
    @api recordId;          // Public property
    @track privateData;     // Reactive private property
    
    // Use @wire for reactive data
    @wire(wireAdapter, { params })
    wiredProperty;
}
```

### 2. Lightning Input Example

**CORRECT:**
```html
<template>
    <lightning-card title="User Form">
        <div class="slds-p-around_medium">
            <lightning-input 
                label="Name" 
                value={name}
                onchange={handleNameChange}>
            </lightning-input>
            
            <lightning-input 
                type="email"
                label="Email" 
                value={email}
                required>
            </lightning-input>
            
            <lightning-button 
                label="Submit" 
                variant="brand"
                onclick={handleSubmit}>
            </lightning-button>
        </div>
    </lightning-card>
</template>
```

**INCORRECT (DO NOT USE):**
```html
<!-- WRONG - Don't use generic HTML -->
<div class="card">
    <input type="text" placeholder="Name" />
    <input type="email" placeholder="Email" />
    <button>Submit</button>
</div>
```

### 3. Lightning Datatable Example

```javascript
// JavaScript
import { LightningElement, wire } from 'lwc';
import getAccounts from '@salesforce/apex/AccountController.getAccounts';

const COLUMNS = [
    { label: 'Account Name', fieldName: 'Name', type: 'text' },
    { label: 'Industry', fieldName: 'Industry', type: 'text' },
    { label: 'Annual Revenue', fieldName: 'AnnualRevenue', type: 'currency' }
];

export default class AccountList extends LightningElement {
    columns = COLUMNS;
    @wire(getAccounts) accounts;
}
```

```html
<!-- HTML -->
<template>
    <lightning-card title="Accounts" icon-name="standard:account">
        <lightning-datatable
            key-field="Id"
            data={accounts.data}
            columns={columns}>
        </lightning-datatable>
    </lightning-card>
</template>
```

### 4. Record Forms

**Quick Record Form (minimal code):**
```html
<lightning-record-form
    object-api-name="Contact"
    fields={fields}
    mode="edit"
    onsubmit={handleSubmit}
    onsuccess={handleSuccess}>
</lightning-record-form>
```

**Custom Record Edit Form (more control):**
```html
<lightning-record-edit-form
    record-id={recordId}
    object-api-name="Account">
    <lightning-messages></lightning-messages>
    <lightning-input-field field-name="Name"></lightning-input-field>
    <lightning-input-field field-name="Industry"></lightning-input-field>
    <lightning-button type="submit" label="Save"></lightning-button>
</lightning-record-edit-form>
```

## Styling with SLDS

Use Salesforce Lightning Design System (SLDS) utility classes instead of custom CSS:

```html
<!-- Use SLDS classes -->
<div class="slds-grid slds-wrap">
    <div class="slds-col slds-size_1-of-2 slds-p-around_small">
        <lightning-input label="First Name"></lightning-input>
    </div>
    <div class="slds-col slds-size_1-of-2 slds-p-around_small">
        <lightning-input label="Last Name"></lightning-input>
    </div>
</div>

<!-- Common SLDS classes -->
<!-- Spacing: slds-p-around_small, slds-m-top_medium -->
<!-- Grid: slds-grid, slds-col, slds-size_1-of-2 -->
<!-- Text: slds-text-heading_medium, slds-text-align_center -->
```

## Common Anti-Patterns to Avoid

### ❌ DON'T: Use Ternary Operators in HTML Templates

**CRITICAL**: LWC templates do NOT support ternary operators or complex JavaScript expressions in HTML.

**WRONG:**
```html
<!-- WRONG - Ternary operators are not supported in LWC templates -->
<div>{isActive ? 'Active' : 'Inactive'}</div>
<lightning-input label={isRequired ? 'Required Field' : 'Optional Field'}></lightning-input>
<div class={status === 'complete' ? 'green' : 'red'}></div>
```

**CORRECT:**
```html
<!-- CORRECT - Use if:true/if:false or getters -->
<template if:true={isActive}>
    <div>Active</div>
</template>
<template if:false={isActive}>
    <div>Inactive</div>
</template>

<lightning-input label={fieldLabel}></lightning-input>
<div class={statusClass}></div>
```

```javascript
// JavaScript - Use getters for computed values
export default class MyComponent extends LightningElement {
    isActive = true;
    isRequired = true;
    status = 'complete';
    
    get fieldLabel() {
        return this.isRequired ? 'Required Field' : 'Optional Field';
    }
    
    get statusClass() {
        return this.status === 'complete' ? 'green' : 'red';
    }
}
```

### ❌ DON'T: Use Generic HTML Elements
```html
<!-- WRONG -->
<input type="text" />
<button onclick={handleClick}>Click</button>
<select><option>Item</option></select>
```

### ✅ DO: Use Lightning Components
```html
<!-- CORRECT -->
<lightning-input></lightning-input>
<lightning-button onclick={handleClick} label="Click"></lightning-button>
<lightning-combobox options={options}></lightning-combobox>
```

### ❌ DON'T: Manual Form Validation
```javascript
// WRONG - manual validation
if (!this.template.querySelector('input').value) {
    alert('Required field');
}
```

### ✅ DO: Use Built-in Validation
```javascript
// CORRECT - use reportValidity()
const allValid = [...this.template.querySelectorAll('lightning-input')]
    .reduce((validSoFar, inputCmp) => {
        inputCmp.reportValidity();
        return validSoFar && inputCmp.checkValidity();
    }, true);
```

### ❌ DON'T: Direct DOM Manipulation
```javascript
// WRONG
this.template.querySelector('div').innerHTML = 'Text';
```

### ✅ DO: Use Reactive Properties
```javascript
// CORRECT
@track displayText = 'Text';
```

## Component Communication

### Parent to Child
```javascript
// Parent passes data via public property
<c-child-component record-id={recordId}></c-child-component>

// Child receives via @api
export default class ChildComponent extends LightningElement {
    @api recordId;
}
```

### Child to Parent
```javascript
// Child dispatches event
this.dispatchEvent(new CustomEvent('select', {
    detail: { recordId: this.recordId }
}));

// Parent handles event
<c-child-component onselect={handleSelect}></c-child-component>
```

## Lightning Data Service (LDS)

Use LDS for automatic data caching and synchronization:

```javascript
import { LightningElement, wire, api } from 'lwc';
import { getRecord } from 'lightning/uiRecordApi';

const FIELDS = ['Account.Name', 'Account.Industry'];

export default class AccountDetail extends LightningElement {
    @api recordId;
    
    @wire(getRecord, { recordId: '$recordId', fields: FIELDS })
    account;
}
```

## Navigation

```javascript
import { NavigationMixin } from 'lightning/navigation';

export default class MyComponent extends NavigationMixin(LightningElement) {
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

## Key Reminders

1. **ALWAYS use `lightning-*` components** - Never use generic HTML form elements
2. **Use SLDS classes** for styling - Avoid custom CSS when possible
3. **Leverage LDS** for data operations - Built-in caching and sync
4. **Use `@wire`** for reactive data - Automatic re-rendering
5. **Report validity** - Use built-in validation methods
6. **Import only what you need** - Keep bundle size small
7. **Follow naming conventions** - camelCase for properties, kebab-case for HTML attributes
8. **Test with Salesforce data** - Component behavior may differ with real records

## File Structure

```
lwcComponentName/
├── lwcComponentName.html          # Template
├── lwcComponentName.js            # JavaScript controller
├── lwcComponentName.js-meta.xml   # Configuration
└── lwcComponentName.css           # Styles (optional)
```

## Metadata Configuration (.js-meta.xml)

```xml
<?xml version="1.0" encoding="UTF-8"?>
<LightningComponentBundle xmlns="http://soap.sforce.com/2006/04/metadata">
    <apiVersion>60.0</apiVersion>
    <isExposed>true</isExposed>
    <targets>
        <target>lightning__AppPage</target>
        <target>lightning__RecordPage</target>
        <target>lightning__HomePage</target>
    </targets>
    <targetConfigs>
        <targetConfig targets="lightning__RecordPage">
            <objects>
                <object>Account</object>
                <object>Contact</object>
            </objects>
        </targetConfig>
    </targetConfigs>
</LightningComponentBundle>
```

## Quick Reference: Most Common Components

**Forms & Input:**
- `lightning-input` - Text, email, number, date, checkbox, toggle
- `lightning-textarea` - Multi-line text
- `lightning-combobox` - Searchable dropdown
- `lightning-button` - Actions and submissions

**Layout:**
- `lightning-card` - Content container
- `lightning-layout` + `lightning-layout-item` - Responsive grid
- `lightning-accordion` - Collapsible sections

**Data Display:**
- `lightning-datatable` - Tables with sorting/selection
- `lightning-formatted-*` - Display formatted values
- `lightning-icon` - SLDS icons

**Record Operations:**
- `lightning-record-form` - Quick create/edit/view
- `lightning-record-edit-form` - Custom edit forms
- `lightning-record-view-form` - Display records

**Feedback:**
- `lightning-spinner` - Loading indicator
- `lightning-messages` - Form validation messages

Remember: When in doubt, search for "lightning-" + functionality in the Salesforce Component Library. The lightning namespace has a component for almost everything!
