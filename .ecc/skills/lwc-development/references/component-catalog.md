# Lightning Web Components Catalog

Complete reference of standard Lightning components organized by category.

## Input Components

### lightning-input
Multi-purpose input field supporting various types.

**Supported Types:**
- text, email, password, tel, url
- number, date, datetime, time
- checkbox, checkbox-button, toggle
- file, color, search

**Key Attributes:**
- `label` - Field label (required for accessibility)
- `value` - Current value
- `type` - Input type (default: text)
- `required` - Makes field required
- `disabled` - Disables input
- `readonly` - Makes field read-only
- `placeholder` - Placeholder text
- `min` / `max` - Min/max values (for number, date)
- `pattern` - Regex validation pattern
- `message-when-*` - Custom validation messages

**Example:**
```html
<lightning-input 
    type="email"
    label="Email Address" 
    value={email}
    required
    onchange={handleChange}>
</lightning-input>
```

### lightning-textarea
Multi-line text input.

**Key Attributes:**
- `label`, `value`, `placeholder`
- `required`, `disabled`, `readonly`
- `max-length` - Character limit

**Example:**
```html
<lightning-textarea
    label="Description"
    value={description}
    max-length="500">
</lightning-textarea>
```

### lightning-combobox
Searchable dropdown with single selection.

**Key Attributes:**
- `label`, `value`, `placeholder`
- `options` - Array of {label, value} objects
- `required`, `disabled`
- `variant` - label-hidden, label-inline, label-stacked

**Example:**
```javascript
// JavaScript
options = [
    { label: 'Option 1', value: 'opt1' },
    { label: 'Option 2', value: 'opt2' }
];
```
```html
<!-- HTML -->
<lightning-combobox
    label="Status"
    value={selectedStatus}
    options={options}
    onchange={handleChange}>
</lightning-combobox>
```

### lightning-dual-listbox
Move items between two lists (available vs selected).

**Example:**
```html
<lightning-dual-listbox
    label="Select Options"
    source-label="Available"
    selected-label="Selected"
    options={allOptions}
    value={selectedValues}>
</lightning-dual-listbox>
```

### lightning-slider
Visual numeric range slider.

**Key Attributes:**
- `label`, `value`
- `min`, `max`, `step`

**Example:**
```html
<lightning-slider
    label="Volume"
    value={volume}
    min="0"
    max="100"
    step="5">
</lightning-slider>
```

### lightning-radio-group
Radio button group for single selection.

**Example:**
```javascript
options = [
    { label: 'Daily', value: 'daily' },
    { label: 'Weekly', value: 'weekly' },
    { label: 'Monthly', value: 'monthly' }
];
```
```html
<lightning-radio-group
    label="Frequency"
    options={options}
    value={selectedFrequency}
    type="radio">
</lightning-radio-group>
```

### lightning-checkbox-group
Multiple checkbox selection.

**Example:**
```html
<lightning-checkbox-group
    label="Interests"
    options={interests}
    value={selectedInterests}>
</lightning-checkbox-group>
```

### lightning-input-address
Compound address field with street, city, state, zip, country.

**Key Attributes:**
- Individual field attributes: `street`, `city`, `province`, `postal-code`, `country`
- `address-label` - Label for entire component
- `street-label`, `city-label`, etc. - Individual field labels
- `required` - Makes all fields required
- `disabled` - Disables all fields

**Example:**
```html
<lightning-input-address
    address-label="Billing Address"
    street={street}
    city={city}
    province={state}
    postal-code={zip}
    country={country}
    required>
</lightning-input-address>
```

### lightning-input-location
Geographic location with latitude/longitude.

**Example:**
```html
<lightning-input-location
    label="Event Location"
    latitude={latitude}
    longitude={longitude}>
</lightning-input-location>
```

## Button Components

### lightning-button
Standard button for actions.

**Variants:**
- `base` - Default
- `neutral` - Gray
- `brand` - Primary brand color
- `destructive` - Red (for delete actions)
- `success` - Green

**Key Attributes:**
- `label` - Button text
- `variant` - Visual style
- `type` - button, submit, reset
- `disabled`
- `icon-name` - SLDS icon
- `icon-position` - left, right

**Example:**
```html
<lightning-button
    label="Save"
    variant="brand"
    type="submit"
    icon-name="utility:save"
    onclick={handleSave}>
</lightning-button>
```

### lightning-button-icon
Icon-only button.

**Key Attributes:**
- `icon-name` - SLDS icon (required)
- `variant` - bare, container, brand, etc.
- `alternative-text` - Screen reader text (required)
- `size` - xx-small, x-small, small, medium, large

**Example:**
```html
<lightning-button-icon
    icon-name="utility:edit"
    variant="bare"
    alternative-text="Edit"
    onclick={handleEdit}>
</lightning-button-icon>
```

### lightning-button-menu
Dropdown menu with actions.

**Example:**
```javascript
menuItems = [
    { label: 'Edit', value: 'edit' },
    { label: 'Delete', value: 'delete' }
];
```
```html
<lightning-button-menu
    alternative-text="Actions"
    icon-size="x-small"
    onselect={handleMenuSelect}>
    <lightning-menu-item value="edit" label="Edit"></lightning-menu-item>
    <lightning-menu-item value="delete" label="Delete"></lightning-menu-item>
</lightning-button-menu>
```

### lightning-button-group
Group related buttons.

**Example:**
```html
<lightning-button-group>
    <lightning-button label="Approve"></lightning-button>
    <lightning-button label="Reject"></lightning-button>
    <lightning-button label="Defer"></lightning-button>
</lightning-button-group>
```

## Layout Components

### lightning-card
Container with header, body, and footer.

**Key Attributes:**
- `title` - Card header
- `icon-name` - Header icon
- `variant` - narrow (less padding)

**Slots:**
- `actions` - Header action buttons
- `footer` - Footer content

**Example:**
```html
<lightning-card title="Account Details" icon-name="standard:account">
    <div slot="actions">
        <lightning-button label="Edit"></lightning-button>
    </div>
    <div class="slds-p-around_medium">
        Card content here
    </div>
    <div slot="footer">
        Footer content
    </div>
</lightning-card>
```

### lightning-layout & lightning-layout-item
Responsive grid system.

**Key Attributes (lightning-layout):**
- `multiple-rows` - Allow wrapping
- `horizontal-align` - spread, space, center, end
- `vertical-align` - start, center, end, stretch

**Key Attributes (lightning-layout-item):**
- `size` - 1-12 (of 12 columns)
- `small-device-size`, `medium-device-size`, `large-device-size`
- `flexibility` - auto, shrink, no-shrink, grow, no-grow
- `padding` - around-small, around-medium, horizontal-small, etc.

**Example:**
```html
<lightning-layout multiple-rows>
    <lightning-layout-item size="6" padding="around-small">
        <lightning-input label="First Name"></lightning-input>
    </lightning-layout-item>
    <lightning-layout-item size="6" padding="around-small">
        <lightning-input label="Last Name"></lightning-input>
    </lightning-layout-item>
</lightning-layout>
```

### lightning-accordion & lightning-accordion-section
Collapsible sections.

**Example:**
```html
<lightning-accordion active-section-name="A">
    <lightning-accordion-section name="A" label="Section A">
        Content for section A
    </lightning-accordion-section>
    <lightning-accordion-section name="B" label="Section B">
        Content for section B
    </lightning-accordion-section>
</lightning-accordion>
```

### lightning-tab & lightning-tabset
Tabbed interface.

**Example:**
```html
<lightning-tabset active-tab-value="one">
    <lightning-tab label="Tab 1" value="one">
        Content for tab 1
    </lightning-tab>
    <lightning-tab label="Tab 2" value="two">
        Content for tab 2
    </lightning-tab>
</lightning-tabset>
```

### lightning-tile
Compact display of records or items.

**Example:**
```html
<lightning-tile label="Contact Name" href="/contact/123">
    <div class="slds-tile__detail">
        <p>Title: Manager</p>
        <p>Phone: 555-1234</p>
    </div>
</lightning-tile>
```

## Data Display Components

### lightning-datatable
Full-featured data table.

**Key Features:**
- Sorting
- Row selection
- Inline editing
- Custom cell types
- Pagination support

**Column Types:**
- text, number, currency, percent
- date, date-local
- email, phone, url
- boolean
- button, button-icon
- action

**Example:**
```javascript
columns = [
    { label: 'Name', fieldName: 'name', type: 'text', sortable: true },
    { label: 'Amount', fieldName: 'amount', type: 'currency', 
      typeAttributes: { currencyCode: 'USD' } },
    { label: 'Status', fieldName: 'status', type: 'text' },
    { type: 'action', typeAttributes: { rowActions: this.rowActions } }
];

rowActions = [
    { label: 'Edit', name: 'edit' },
    { label: 'Delete', name: 'delete' }
];
```
```html
<lightning-datatable
    key-field="id"
    data={data}
    columns={columns}
    onsort={handleSort}
    onrowaction={handleRowAction}
    sorted-by={sortedBy}
    sorted-direction={sortedDirection}>
</lightning-datatable>
```

### lightning-tree
Hierarchical tree structure.

**Example:**
```javascript
treeItems = [
    {
        label: 'Parent 1',
        name: 'p1',
        expanded: true,
        items: [
            { label: 'Child 1', name: 'c1' },
            { label: 'Child 2', name: 'c2' }
        ]
    }
];
```
```html
<lightning-tree items={treeItems} onselect={handleSelect}></lightning-tree>
```

### lightning-tree-grid
Tree structure with columns.

**Example:**
```javascript
columns = [
    { label: 'Name', fieldName: 'name', type: 'text' },
    { label: 'Type', fieldName: 'type', type: 'text' }
];

treeData = [
    {
        name: 'Parent',
        type: 'Folder',
        _children: [
            { name: 'Child', type: 'File' }
        ]
    }
];
```
```html
<lightning-tree-grid
    columns={columns}
    data={treeData}
    key-field="name">
</lightning-tree-grid>
```

### Formatted Output Components

All formatted components automatically format values for display.

- `lightning-formatted-date-time` - Formats date/time
- `lightning-formatted-email` - Email with mailto link
- `lightning-formatted-location` - Lat/long with map link
- `lightning-formatted-name` - Formatted person name
- `lightning-formatted-number` - Number with locale formatting
- `lightning-formatted-phone` - Phone with tel link
- `lightning-formatted-rich-text` - Rich text content
- `lightning-formatted-text` - Plain text with line breaks
- `lightning-formatted-time` - Time value
- `lightning-formatted-url` - URL with link
- `lightning-formatted-address` - Formatted address

**Examples:**
```html
<lightning-formatted-date-time value={dateValue} 
    year="numeric" month="long" day="numeric">
</lightning-formatted-date-time>

<lightning-formatted-email value={emailAddress}></lightning-formatted-email>

<lightning-formatted-number value={amount} 
    format-style="currency" currency-code="USD">
</lightning-formatted-number>

<lightning-formatted-phone value={phoneNumber}></lightning-formatted-phone>

<lightning-formatted-url value={websiteUrl}></lightning-formatted-url>
```

### lightning-icon
Display SLDS icons.

**Icon Categories:**
- standard (objects: account, contact, etc.)
- custom (custom objects)
- utility (actions, symbols)
- doctype (file types)
- action (action icons)

**Key Attributes:**
- `icon-name` - Format: category:name (e.g., "utility:save")
- `size` - xx-small, x-small, small, medium, large
- `variant` - error, warning, success, inverse

**Example:**
```html
<lightning-icon 
    icon-name="standard:account" 
    size="small"
    alternative-text="Account">
</lightning-icon>
```

### lightning-avatar
User profile image.

**Variants:**
- circle (default)
- square

**Sizes:**
- x-small, small, medium (default), large

**Example:**
```html
<lightning-avatar
    src={photoUrl}
    fallback-icon-name="standard:user"
    alternative-text={userName}
    size="medium">
</lightning-avatar>
```

### lightning-badge
Status indicator label.

**Example:**
```html
<lightning-badge label="New"></lightning-badge>
<lightning-badge label="In Progress"></lightning-badge>
```

### lightning-progress-bar
Visual progress indicator.

**Key Attributes:**
- `value` - Progress percentage (0-100)
- `variant` - base (default), circular

**Example:**
```html
<lightning-progress-bar value={progressValue}></lightning-progress-bar>
```

### lightning-progress-indicator
Multi-step process indicator.

**Types:**
- base - Horizontal path
- path - Legacy path style

**Example:**
```html
<lightning-progress-indicator 
    current-step={currentStep} 
    type="base"
    variant="base">
    <lightning-progress-step label="Step 1" value="1"></lightning-progress-step>
    <lightning-progress-step label="Step 2" value="2"></lightning-progress-step>
    <lightning-progress-step label="Step 3" value="3"></lightning-progress-step>
</lightning-progress-indicator>
```

## Record Interaction Components

### lightning-record-form
Simplified form for create/edit/view with automatic layout.

**Modes:**
- `view` - Read-only
- `edit` - Edit existing record
- `create` - Create new record (no recordId)

**Key Attributes:**
- `record-id` - Record to display/edit
- `object-api-name` - Object type
- `fields` - Array of field API names
- `layout-type` - Full, Compact
- `mode` - view, edit, create
- `columns` - Number of columns (1 or 2)

**Events:**
- `onload`, `onsubmit`, `onsuccess`, `onerror`

**Example:**
```javascript
fields = ['Name', 'Industry', 'AnnualRevenue'];
```
```html
<lightning-record-form
    record-id={recordId}
    object-api-name="Account"
    fields={fields}
    mode="edit"
    columns="2"
    onsuccess={handleSuccess}>
</lightning-record-form>
```

### lightning-record-edit-form
Custom form with full control over layout and fields.

**Example:**
```html
<lightning-record-edit-form
    record-id={recordId}
    object-api-name="Contact">
    <lightning-messages></lightning-messages>
    
    <div class="slds-grid slds-gutters">
        <div class="slds-col">
            <lightning-input-field field-name="FirstName"></lightning-input-field>
        </div>
        <div class="slds-col">
            <lightning-input-field field-name="LastName"></lightning-input-field>
        </div>
    </div>
    
    <lightning-input-field field-name="Email"></lightning-input-field>
    <lightning-input-field field-name="Phone"></lightning-input-field>
    
    <div class="slds-m-top_medium">
        <lightning-button type="submit" label="Save"></lightning-button>
        <lightning-button label="Cancel" onclick={handleCancel}></lightning-button>
    </div>
</lightning-record-edit-form>
```

### lightning-record-view-form
Display record fields in read-only mode.

**Example:**
```html
<lightning-record-view-form
    record-id={recordId}
    object-api-name="Account">
    <div class="slds-grid slds-gutters">
        <div class="slds-col">
            <lightning-output-field field-name="Name"></lightning-output-field>
            <lightning-output-field field-name="Industry"></lightning-output-field>
        </div>
        <div class="slds-col">
            <lightning-output-field field-name="AnnualRevenue"></lightning-output-field>
            <lightning-output-field field-name="Phone"></lightning-output-field>
        </div>
    </div>
</lightning-record-view-form>
```

### lightning-input-field
Used within record-edit-form for field input.

**Example:**
```html
<lightning-input-field 
    field-name="CustomField__c"
    value={fieldValue}
    onchange={handleChange}>
</lightning-input-field>
```

### lightning-output-field
Used within record-view-form for field display.

**Example:**
```html
<lightning-output-field field-name="Name"></lightning-output-field>
```

## Navigation & Feedback Components

### lightning-spinner
Loading indicator.

**Variants:**
- base (default)
- brand
- inverse

**Sizes:**
- small, medium (default), large

**Example:**
```html
<lightning-spinner 
    alternative-text="Loading" 
    size="medium"
    if:true={isLoading}>
</lightning-spinner>
```

### lightning-messages
Display form-level validation messages.

**Example:**
```html
<lightning-record-edit-form>
    <lightning-messages></lightning-messages>
    <!-- form fields -->
</lightning-record-edit-form>
```

### lightning-pill
Removable tag/label.

**Example:**
```html
<lightning-pill 
    label={tagName}
    onremove={handleRemove}
    href={recordUrl}>
</lightning-pill>
```

### lightning-pill-container
Container for multiple pills.

**Example:**
```html
<lightning-pill-container items={selectedItems}></lightning-pill-container>
```

### lightning-helptext
Tooltip help text.

**Example:**
```html
<lightning-helptext content="This field is required for all accounts.">
</lightning-helptext>
```

### lightning-breadcrumb & lightning-breadcrumbs
Navigation hierarchy.

**Example:**
```html
<lightning-breadcrumbs>
    <lightning-breadcrumb label="Home" href="/"></lightning-breadcrumb>
    <lightning-breadcrumb label="Accounts" href="/accounts"></lightning-breadcrumb>
    <lightning-breadcrumb label="Acme Corp"></lightning-breadcrumb>
</lightning-breadcrumbs>
```

## Special Components

### lightning-map
Display map with markers.

**Example:**
```javascript
mapMarkers = [
    {
        location: {
            Street: '1 Market St',
            City: 'San Francisco',
            State: 'CA'
        },
        title: 'Office Location'
    }
];
```
```html
<lightning-map map-markers={mapMarkers}></lightning-map>
```

### lightning-file-upload
File upload control.

**Key Attributes:**
- `accept` - File types to accept
- `multiple` - Allow multiple files
- `disabled`
- `label`

**Example:**
```html
<lightning-file-upload
    label="Upload Files"
    accept=".pdf,.jpg,.png"
    multiple
    onuploadfinished={handleUploadFinished}>
</lightning-file-upload>
```

### lightning-carousel
Image carousel.

**Example:**
```html
<lightning-carousel>
    <lightning-carousel-image
        src={image1Url}
        header="Image 1"
        description="First image"
        alternative-text="Image 1">
    </lightning-carousel-image>
    <lightning-carousel-image
        src={image2Url}
        header="Image 2"
        alternative-text="Image 2">
    </lightning-carousel-image>
</lightning-carousel>
```

### lightning-vertical-navigation
Side navigation menu.

**Example:**
```html
<lightning-vertical-navigation selected-item={selectedItem}>
    <lightning-vertical-navigation-section label="Reports">
        <lightning-vertical-navigation-item
            label="Recent"
            name="recent">
        </lightning-vertical-navigation-item>
        <lightning-vertical-navigation-item
            label="Created by Me"
            name="created">
        </lightning-vertical-navigation-item>
    </lightning-vertical-navigation-section>
</lightning-vertical-navigation>
```

## Component Selection Decision Tree

When choosing a component, follow this decision tree:

1. **Collecting user input?**
   - Single-line text → `lightning-input`
   - Multi-line text → `lightning-textarea`
   - Dropdown → `lightning-combobox`
   - Date/time → `lightning-input type="date"`

2. **Displaying data?**
   - Table → `lightning-datatable`
   - Single formatted value → `lightning-formatted-*`
   - Hierarchy → `lightning-tree`

3. **User action needed?**
   - Simple action → `lightning-button`
   - Icon action → `lightning-button-icon`
   - Multiple actions → `lightning-button-menu`

4. **Working with Salesforce records?**
   - Quick form → `lightning-record-form`
   - Custom form → `lightning-record-edit-form`
   - Display only → `lightning-record-view-form`

5. **Layout needed?**
   - Container → `lightning-card`
   - Grid → `lightning-layout`
   - Tabs → `lightning-tabset`
   - Sections → `lightning-accordion`

6. **Providing feedback?**
   - Loading → `lightning-spinner`
   - Status → `lightning-badge`
   - Progress → `lightning-progress-bar`

Remember: If there's a `lightning-*` component for your use case, use it instead of building custom HTML!
