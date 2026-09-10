# LWC Common Patterns & Examples

Quick reference for frequently used Lightning Web Component patterns.

## Form Validation Pattern

```javascript
// JavaScript
handleSubmit() {
    // Validate all lightning-input fields
    const allValid = [...this.template.querySelectorAll('lightning-input')]
        .reduce((validSoFar, inputCmp) => {
            inputCmp.reportValidity();
            return validSoFar && inputCmp.checkValidity();
        }, true);
    
    if (allValid) {
        // Process form
        this.saveData();
    }
}
```

## Wire Service Pattern

```javascript
import { LightningElement, wire, api } from 'lwc';
import { getRecord, getFieldValue } from 'lightning/uiRecordApi';
import NAME_FIELD from '@salesforce/schema/Account.Name';

export default class AccountName extends LightningElement {
    @api recordId;
    
    @wire(getRecord, { recordId: '$recordId', fields: [NAME_FIELD] })
    account;
    
    get accountName() {
        return getFieldValue(this.account.data, NAME_FIELD);
    }
}
```

## Imperative Apex Call Pattern

```javascript
import { LightningElement } from 'lwc';
import { ShowToastEvent } from 'lightning/platformShowToastEvent';
import getAccountList from '@salesforce/apex/AccountController.getAccountList';

export default class AccountList extends LightningElement {
    accounts;
    error;
    
    connectedCallback() {
        this.loadAccounts();
    }
    
    loadAccounts() {
        getAccountList()
            .then(result => {
                this.accounts = result;
                this.error = undefined;
            })
            .catch(error => {
                this.error = error;
                this.accounts = undefined;
                this.showToast('Error', error.body.message, 'error');
            });
    }
    
    showToast(title, message, variant) {
        this.dispatchEvent(new ShowToastEvent({ title, message, variant }));
    }
}
```

## Datatable with Row Actions Pattern

```javascript
const COLUMNS = [
    { label: 'Name', fieldName: 'Name', type: 'text' },
    { label: 'Industry', fieldName: 'Industry', type: 'text' },
    { 
        type: 'action', 
        typeAttributes: { 
            rowActions: [
                { label: 'Edit', name: 'edit' },
                { label: 'Delete', name: 'delete' }
            ]
        }
    }
];

export default class AccountTable extends LightningElement {
    columns = COLUMNS;
    @wire(getAccounts) accounts;
    
    handleRowAction(event) {
        const actionName = event.detail.action.name;
        const row = event.detail.row;
        
        switch (actionName) {
            case 'edit':
                this.editRecord(row);
                break;
            case 'delete':
                this.deleteRecord(row);
                break;
        }
    }
    
    editRecord(row) {
        // Handle edit
    }
    
    deleteRecord(row) {
        // Handle delete
    }
}
```

## Navigation Pattern

```javascript
import { NavigationMixin } from 'lightning/navigation';

export default class MyComponent extends NavigationMixin(LightningElement) {
    
    // Navigate to record
    navigateToRecord(recordId) {
        this[NavigationMixin.Navigate]({
            type: 'standard__recordPage',
            attributes: {
                recordId: recordId,
                objectApiName: 'Account',
                actionName: 'view'
            }
        });
    }
    
    // Navigate to list view
    navigateToListView() {
        this[NavigationMixin.Navigate]({
            type: 'standard__objectPage',
            attributes: {
                objectApiName: 'Account',
                actionName: 'list'
            }
        });
    }
    
    // Navigate to create new record
    navigateToNew() {
        this[NavigationMixin.Navigate]({
            type: 'standard__objectPage',
            attributes: {
                objectApiName: 'Contact',
                actionName: 'new'
            }
        });
    }
    
    // Navigate to URL
    navigateToUrl(url) {
        this[NavigationMixin.Navigate]({
            type: 'standard__webPage',
            attributes: {
                url: url
            }
        });
    }
}
```

## Parent-Child Communication Pattern

```javascript
// PARENT COMPONENT
// parent.html
<template>
    <c-child-component 
        data={parentData}
        onchildevent={handleChildEvent}>
    </c-child-component>
</template>

// parent.js
export default class Parent extends LightningElement {
    parentData = { name: 'Test', value: 123 };
    
    handleChildEvent(event) {
        const dataFromChild = event.detail;
        console.log('Received from child:', dataFromChild);
    }
}

// CHILD COMPONENT
// child.js
import { LightningElement, api } from 'lwc';

export default class Child extends LightningElement {
    @api data; // Receives data from parent
    
    sendToParent() {
        // Send data to parent via custom event
        const event = new CustomEvent('childevent', {
            detail: { message: 'Hello from child', value: 456 }
        });
        this.dispatchEvent(event);
    }
}
```

## Conditional Rendering Pattern

```html
<template>
    <!-- Show/hide based on boolean -->
    <div if:true={isLoading}>
        <lightning-spinner></lightning-spinner>
    </div>
    
    <div if:false={isLoading}>
        <!-- Content when not loading -->
    </div>
    
    <!-- Iterate over list -->
    <template for:each={items} for:item="item">
        <div key={item.Id}>
            {item.Name}
        </div>
    </template>
    
    <!-- Iterate with index -->
    <template for:each={items} for:item="item" for:index="index">
        <div key={item.Id}>
            {index}: {item.Name}
        </div>
    </template>
    
    <!-- Iterator (alternative to for:each) -->
    <template iterator:it={items}>
        <div key={it.value.Id}>
            <span if:true={it.first}>First Item: </span>
            <span if:true={it.last}>Last Item: </span>
            {it.value.Name}
        </div>
    </template>
</template>
```

## Refresh Data Pattern

```javascript
import { LightningElement, wire } from 'lwc';
import { refreshApex } from '@salesforce/apex';
import getAccounts from '@salesforce/apex/AccountController.getAccounts';

export default class RefreshableList extends LightningElement {
    wiredAccountsResult;
    
    @wire(getAccounts)
    wiredAccounts(result) {
        this.wiredAccountsResult = result;
        if (result.data) {
            this.accounts = result.data;
        }
    }
    
    handleRefresh() {
        // Refresh wired data
        return refreshApex(this.wiredAccountsResult);
    }
}
```

## Toast Notification Pattern

```javascript
import { ShowToastEvent } from 'lightning/platformShowToastEvent';

export default class MyComponent extends LightningElement {
    
    showSuccess(message) {
        this.dispatchEvent(new ShowToastEvent({
            title: 'Success',
            message: message,
            variant: 'success'
        }));
    }
    
    showError(message) {
        this.dispatchEvent(new ShowToastEvent({
            title: 'Error',
            message: message,
            variant: 'error',
            mode: 'sticky' // Stays until closed
        }));
    }
    
    showWarning(message) {
        this.dispatchEvent(new ShowToastEvent({
            title: 'Warning',
            message: message,
            variant: 'warning'
        }));
    }
    
    showInfo(message) {
        this.dispatchEvent(new ShowToastEvent({
            title: 'Info',
            message: message,
            variant: 'info'
        }));
    }
}
```

## Modal/Popup Pattern

```html
<!-- modal.html -->
<template>
    <section if:true={isOpen} role="dialog" class="slds-modal slds-fade-in-open">
        <div class="slds-modal__container">
            <header class="slds-modal__header">
                <button class="slds-button slds-button_icon slds-modal__close" 
                        onclick={closeModal}>
                    <lightning-icon icon-name="utility:close" 
                                    alternative-text="close" 
                                    size="small">
                    </lightning-icon>
                </button>
                <h2 class="slds-text-heading_medium">{title}</h2>
            </header>
            <div class="slds-modal__content slds-p-around_medium">
                <slot></slot>
            </div>
            <footer class="slds-modal__footer">
                <lightning-button label="Cancel" onclick={closeModal}></lightning-button>
                <lightning-button variant="brand" label="Save" onclick={handleSave}></lightning-button>
            </footer>
        </div>
    </section>
    <div if:true={isOpen} class="slds-backdrop slds-backdrop_open"></div>
</template>
```

```javascript
// modal.js
import { LightningElement, api } from 'lwc';

export default class Modal extends LightningElement {
    @api title = 'Modal';
    isOpen = false;
    
    @api
    open() {
        this.isOpen = true;
    }
    
    @api
    close() {
        this.isOpen = false;
    }
    
    closeModal() {
        this.close();
    }
    
    handleSave() {
        // Handle save logic
        this.dispatchEvent(new CustomEvent('save'));
        this.close();
    }
}
```

## Dynamic Styling Pattern

```html
<template>
    <div class={dynamicClass} style={dynamicStyle}>
        Content
    </div>
</template>
```

```javascript
export default class DynamicStyle extends LightningElement {
    isActive = true;
    backgroundColor = '#f0f0f0';
    
    get dynamicClass() {
        return this.isActive ? 'slds-box active' : 'slds-box';
    }
    
    get dynamicStyle() {
        return `background-color: ${this.backgroundColor}; padding: 1rem;`;
    }
}
```

## Debounce Search Pattern

```javascript
export default class SearchComponent extends LightningElement {
    searchTerm = '';
    searchResults = [];
    
    handleSearchTermChange(event) {
        // Clear previous timeout
        window.clearTimeout(this.delayTimeout);
        
        const searchTerm = event.target.value;
        
        // Debounce search
        this.delayTimeout = setTimeout(() => {
            this.searchTerm = searchTerm;
            this.performSearch();
        }, 300); // Wait 300ms after user stops typing
    }
    
    performSearch() {
        if (this.searchTerm.length >= 2) {
            // Execute search
            searchRecords({ searchTerm: this.searchTerm })
                .then(result => {
                    this.searchResults = result;
                })
                .catch(error => {
                    console.error(error);
                });
        }
    }
}
```

## Getter/Computed Property Pattern

```javascript
export default class ComputedExample extends LightningElement {
    firstName = 'John';
    lastName = 'Doe';
    items = [];
    isActive = true;
    status = 'pending';
    
    // Computed property - recalculated when dependencies change
    get fullName() {
        return `${this.firstName} ${this.lastName}`;
    }
    
    get hasItems() {
        return this.items && this.items.length > 0;
    }
    
    get itemCount() {
        return this.items ? this.items.length : 0;
    }
    
    get formattedItems() {
        return this.items.map((item, index) => ({
            ...item,
            displayName: `${index + 1}. ${item.name}`
        }));
    }
    
    // CRITICAL: Use getters for conditional display text
    // Ternary operators NOT supported directly in templates!
    get statusText() {
        return this.isActive ? 'Active' : 'Inactive';
    }
    
    get statusClass() {
        return this.status === 'complete' 
            ? 'slds-text-color_success' 
            : 'slds-text-color_error';
    }
    
    get displayCount() {
        return this.itemCount > 0 ? `${this.itemCount} items` : 'No items';
    }
}
```

```html
<template>
    <!-- ✅ CORRECT - Use getters for conditional values -->
    <div>{statusText}</div>
    <div class={statusClass}>{displayCount}</div>
    
    <!-- ❌ WRONG - Ternary operators not supported in templates -->
    <!-- <div>{isActive ? 'Active' : 'Inactive'}</div> -->
</template>
```

## Error Handling Pattern

```javascript
import { LightningElement } from 'lwc';
import { ShowToastEvent } from 'lightning/platformShowToastEvent';
import performAction from '@salesforce/apex/MyController.performAction';

export default class ErrorHandling extends LightningElement {
    isLoading = false;
    
    async handleAction() {
        this.isLoading = true;
        
        try {
            const result = await performAction({ param: 'value' });
            this.handleSuccess(result);
        } catch (error) {
            this.handleError(error);
        } finally {
            this.isLoading = false;
        }
    }
    
    handleSuccess(result) {
        this.dispatchEvent(new ShowToastEvent({
            title: 'Success',
            message: 'Action completed successfully',
            variant: 'success'
        }));
    }
    
    handleError(error) {
        let message = 'Unknown error';
        
        if (Array.isArray(error.body)) {
            message = error.body.map(e => e.message).join(', ');
        } else if (error.body && error.body.message) {
            message = error.body.message;
        } else if (error.message) {
            message = error.message;
        }
        
        this.dispatchEvent(new ShowToastEvent({
            title: 'Error',
            message: message,
            variant: 'error',
            mode: 'sticky'
        }));
    }
}
```

## Lifecycle Hooks Pattern

```javascript
import { LightningElement } from 'lwc';

export default class LifecycleExample extends LightningElement {
    
    // Called when component is inserted into DOM
    connectedCallback() {
        console.log('Component connected');
        // Good place for: initial data loading, event listeners
    }
    
    // Called when component is removed from DOM
    disconnectedCallback() {
        console.log('Component disconnected');
        // Good place for: cleanup, removing event listeners
    }
    
    // Called after every render
    renderedCallback() {
        console.log('Component rendered');
        // Good place for: DOM manipulation (use sparingly)
        // Called multiple times - be careful with side effects
    }
    
    // Called when public property value changes
    // Not needed for @api properties with getters/setters
    
    // Called when error occurs in component or its children
    errorCallback(error, stack) {
        console.error('Error occurred:', error);
        console.error('Stack trace:', stack);
        // Good place for: error logging, error handling
    }
}
```

## RecordId Context Pattern (for Record Pages)

```javascript
import { LightningElement, api } from 'lwc';

export default class RecordPageComponent extends LightningElement {
    @api recordId; // Automatically populated when on a record page
    
    connectedCallback() {
        if (this.recordId) {
            console.log('Current record ID:', this.recordId);
            // Use recordId to load related data
        }
    }
}
```

## Quick Action Pattern

```xml
<!-- Metadata file for quick action -->
<?xml version="1.0" encoding="UTF-8"?>
<LightningComponentBundle xmlns="http://soap.sforce.com/2006/04/metadata">
    <apiVersion>60.0</apiVersion>
    <isExposed>true</isExposed>
    <targets>
        <target>lightning__RecordAction</target>
    </targets>
    <targetConfigs>
        <targetConfig targets="lightning__RecordAction">
            <actionType>ScreenAction</actionType>
        </targetConfig>
    </targetConfigs>
</LightningComponentBundle>
```

```javascript
// Component for quick action
import { LightningElement, api } from 'lwc';
import { CloseActionScreenEvent } from 'lightning/actions';

export default class QuickAction extends LightningElement {
    @api recordId;
    
    handleSuccess() {
        // Close the quick action modal
        this.dispatchEvent(new CloseActionScreenEvent());
    }
    
    handleCancel() {
        this.dispatchEvent(new CloseActionScreenEvent());
    }
}
```

These patterns cover the most common scenarios in LWC development. Remember to always use `lightning-*` components where available rather than implementing these patterns with generic HTML!
