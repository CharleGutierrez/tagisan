---
name: apex-development
description: Expert guidance for Apex development on Salesforce Platform following NimbleUser style
  guide standards. Use this skill when working with Apex classes, triggers, test classes,
  or when the user mentions Salesforce Apex, .cls files, .trigger files, or Apex-specific
  concepts like SOQL, DML, governor limits, or test coverage.
...
---
# Apex Development Style Guide

## Core Principles

**CRITICAL**: Follow the NimbleUser Apex Style Guide (fork of Google Java Style Guide adapted for Salesforce Apex). Write clean, maintainable, and consistent Apex code that adheres to industry best practices.

## Source File Structure

### File Organization

**Required Order:**
1. Top-level ApexDoc comments
2. Class declaration
3. Class members in logical order

**Key Rules:**
- Exactly ONE top-level class per file
- File name MUST match class name with `.cls` extension
- UTF-8 encoding
- NO tab characters (use 4 spaces for indentation)
- ASCII horizontal spaces only

**Example:**
```apex
/**
 * @description Handles account-related business logic
 */
public with sharing class AccountService {
    // Fields
    private static final Integer MAX_RECORDS = 200;
    private AccountRepository repository;

    // Constructors
    public AccountService() {
        this.repository = new AccountRepository();
    }

    // Public methods
    public List<Account> getActiveAccounts() {
        return repository.findActiveAccounts();
    }

    // Private methods
    private void validateAccount(Account acc) {
        // validation logic
    }
}
```

## Formatting Standards

### Braces (K&R Style)

**Always Required** for `if`, `else`, `for`, `do`, `while` - even for single-line bodies.

**Rules:**
- Opening brace on SAME line as statement
- Line break AFTER opening brace
- Line break BEFORE closing brace
- Closing brace on its own line

**CORRECT:**
```apex
// ✅ CORRECT
if (isValid) {
    processRecord();
}

if (count > 0) {
    updateRecords();
} else {
    createRecords();
}

for (Account acc : accounts) {
    acc.Status__c = 'Active';
}

// Empty block (allowed)
public void emptyMethod() {}
```

**WRONG:**
```apex
// ❌ WRONG - Missing braces
if (isValid)
    processRecord();

// ❌ WRONG - Opening brace on new line
if (isValid)
{
    processRecord();
}

// ❌ WRONG - No braces for single line
for (Account acc : accounts)
    acc.Status__c = 'Active';
```

### Indentation

- Each block level: **+4 spaces**
- Continuation lines: **minimum +4 spaces**
- NO tabs allowed

```apex
public void complexMethod() {
    if (condition) {
        for (Integer i = 0; i < limit; i++) {
            system.debug('Level 3 indent = 12 spaces');
        }
    }
}
```

### Line Length and Wrapping

**Column Limit: 120 characters**

**Breaking Rules:**
- Break at HIGHER syntactic levels
- Non-assignment operators: break BEFORE the symbol
- Assignment operators: break AFTER the symbol (or before, both acceptable)
- Method names stay ATTACHED to opening parenthesis
- Commas stay ATTACHED to preceding token
- Continuation lines: minimum +4 spaces indent

**CORRECT:**
```apex
// ✅ Method call wrapping
Account newAccount = AccountFactory.createAccount(
    accountName,
    industryType,
    annualRevenue
);

// ✅ Long condition wrapping
if (account.AnnualRevenue > THRESHOLD
    && account.Industry == 'Technology'
    && account.Rating == 'Hot') {
    processHighValueAccount(account);
}

// ✅ Assignment wrapping
String longMessage =
    'This is a very long message that exceeds the line limit';
```

### SOQL/SOSL Formatting

**Special Rules for SOQL:**
- Line breaks are OPTIONAL
- Break BEFORE reserved words
- Standard +4 space indentation from containing block
- Reserved words in ALL UPPERCASE

**CORRECT:**
```apex
// ✅ CORRECT - Multi-line SOQL
List<Account> accounts = [
    SELECT Id, Name, Industry, AnnualRevenue,
        (SELECT Id, FirstName, LastName FROM Contacts)
    FROM Account
    WHERE Industry = :targetIndustry
        AND AnnualRevenue > :minRevenue
    ORDER BY Name
    LIMIT 200
];

// ✅ CORRECT - Simple single-line SOQL
List<Account> accounts = [SELECT Id, Name FROM Account WHERE Industry = 'Technology'];

// ✅ CORRECT - Dynamic SOQL
String query =
    'SELECT Id, Name FROM Account ' +
    'WHERE Industry = :industry ' +
    'ORDER BY Name';
```

### Whitespace

**Vertical Whitespace (Blank Lines):**
- Single blank line BETWEEN class members (fields, constructors, methods, nested classes)
- Optional between consecutive fields for logical grouping
- Between statements for logical subsections

**Horizontal Whitespace:**
```apex
// ✅ CORRECT spacing
if (isValid) {           // Space after 'if', before '{'
    x = a + b;           // Space around operators
    y = c * d;           // Space around operators
}

// Comment spacing       // Space after '//'
String name = 'Test';   // Space before comment

// ❌ WRONG spacing
if(isValid){             // No space after 'if'
    x=a+b;               // No space around operators
}
```

**Horizontal Alignment:**
- NOT required
- Permitted but NOT enforced
- Warning: "Creates problems for future maintenance"

```apex
// Optional (not required)
private Integer  x      = 1;
private String   name   = 'Test';
private Boolean  active = true;
```

## Naming Conventions

### Class Names

**UpperCamelCase** - Nouns or noun phrases

**CORRECT:**
```apex
public class AccountService {}
public class OpportunityTriggerHandler {}
public class HttpClient {}
public class XmlParser {}          // XML acronym
```

**Test Classes:**
```apex
public class AccountServiceTest {}
public class OpportunityTriggerHandlerTest {}
```

### Method Names

**lowerCamelCase** - Verbs or verb phrases

**CORRECT:**
```apex
public void processRecords() {}
public Account getAccountById(Id accountId) {}
public Boolean isValidEmail(String email) {}
public void sendEmail() {}
```

**Test Methods:**
```apex
// Pattern: <methodName>_<condition>_<expectation>
@isTest
static void getAccountById_withValidId_returnsAccount() {}

@isTest
static void processRecords_withEmptyList_throwsException() {}
```

### Constant Names

**CONSTANT_CASE** - All uppercase with underscores

**CORRECT:**
```apex
private static final Integer MAX_RETRY_COUNT = 3;
private static final String DEFAULT_COUNTRY = 'USA';
public static final String API_VERSION = '60.0';
```

**Must be:**
- `static final` fields
- Deeply immutable contents
- NOT just "intending to never mutate"

### Field Names

**lowerCamelCase** - Nouns or noun phrases

**CORRECT:**
```apex
private AccountRepository accountRepository;
private Integer retryCount;
private String errorMessage;
```

**Properties - UpperCamelCase:**
```apex
public String AccountName { get; set; }
public Integer RecordCount { get; private set; }
public Boolean IsActive { get; }
```

### Parameter and Local Variable Names

**lowerCamelCase** - Descriptive names

**CORRECT:**
```apex
public void updateAccount(Account accountToUpdate, String newStatus) {
    String previousStatus = accountToUpdate.Status__c;
    Integer recordCount = 0;
}
```

**AVOID:**
- Single-character names (except loop counters)
- Non-descriptive names

### Camel Case Rules

1. Convert phrase to ASCII, remove apostrophes
2. Split on spaces and punctuation
3. Lowercase everything, uppercase first character(s)
4. Join into single identifier

**Examples:**
- "XML HTTP request" → `XmlHttpRequest`
- "new customer ID" → `newCustomerId`
- "supports IPv6 on iOS" → `supportsIpv6OnIos`

## Code Organization

### Class Member Ordering

**Use logical order** that maintainers can explain:

1. Constants
2. Static fields
3. Instance fields
4. Constructors
5. Public methods
6. Private methods
7. Inner classes

**Overloaded Methods:**
- Appear SEQUENTIALLY
- NO intervening code

```apex
public class OrderService {
    // Constants
    private static final Integer MAX_ORDERS = 100;

    // Static fields
    private static OrderRepository repository = new OrderRepository();

    // Instance fields
    private String userId;

    // Constructors
    public OrderService() {
        this(UserInfo.getUserId());
    }

    public OrderService(String userId) {
        this.userId = userId;
    }

    // Public methods
    public List<Order> getOrders() {
        return getOrders(MAX_ORDERS);
    }

    public List<Order> getOrders(Integer limitCount) {
        return repository.findByUserId(userId, limitCount);
    }

    // Private methods
    private void validateOrder(Order ord) {
        // validation
    }
}
```

### Variable Declarations

- ONE variable per declaration
- Declare CLOSE to first use (not at block start)
- Local variables are NOT constants (don't use CONSTANT_CASE)

**CORRECT:**
```apex
public void processAccounts(List<Account> accounts) {
    for (Account acc : accounts) {
        String industry = acc.Industry;  // Declared close to use
        Integer revenue = acc.AnnualRevenue;

        if (revenue > THRESHOLD) {
            String message = 'High value account';  // Declared when needed
            sendNotification(message);
        }
    }
}
```

## Modifiers Order

**Standard Order:**
```apex
private protected public global virtual abstract with sharing without sharing
```

**Examples:**
```apex
public with sharing class AccountService {}
private abstract class BaseHandler {}
public virtual class TriggerHandler {}
global without sharing class IntegrationService {}
```

## ApexDoc Standards

### When Required

**MUST document:**
- ALL `global` and `public` classes
- ALL `global`, `public`, or `protected` members of public classes

**MAY skip:**
- Self-explanatory methods (simple getters/setters)
- Overriding methods (can reference parent)
- Private classes and members (use as needed)

### Format

**At-clauses Order:**
1. `@description`
2. `@param`
3. `@return`
4. `@throws`

**Multi-line continuations: +4 spaces from `@`**

**CORRECT:**
```apex
/**
 * @description Retrieves active accounts for the specified industry
 * @param industry The industry to filter accounts
 * @param minRevenue The minimum annual revenue threshold
 * @return List of active accounts matching criteria, or empty list
 *     if no matches found
 * @throws QueryException if database query fails
 */
public List<Account> getActiveAccounts(String industry, Decimal minRevenue) {
    // implementation
}

/**
 * @description Handles account creation with validation and
 *     duplicate checking
 */
public class AccountCreationService {
    // implementation
}
```

### Summary Fragment

- Brief noun phrase or verb phrase
- NOT complete sentence
- Capitalized and punctuated as sentence
- DO NOT start with "A {@code Foo} is a..." or "This method returns..."

**CORRECT:**
```apex
/** Processes pending orders and sends notifications */
/** Service for managing account lifecycle */
/** Validates email format and domain */
```

**WRONG:**
```apex
/** This method processes pending orders */  // ❌ Don't start with "This method"
/** A service that manages accounts */        // ❌ Don't start with "A"
```

## Testing Best Practices

### Test Class Declaration

```apex
@isTest
private class AccountServiceTest {
    // All test classes are PRIVATE
    // Marked with @isTest annotation
}
```

### Test Method Pattern

```apex
@isTest
static void getAccountById_withValidId_returnsAccount() {
    // Pattern: <methodName>_<condition>_<expectation>

    // Setup
    Account testAccount = TestDataFactory.createAccount('Test Corp');
    insert testAccount;

    // Execute
    Test.startTest();
    Account result = AccountService.getAccountById(testAccount.Id);
    Test.stopTest();

    // Verify
    System.assertNotEquals(null, result, 'Account should be found');
    System.assertEquals('Test Corp', result.Name, 'Account name should match');
}
```

### Test.startTest() and Test.stopTest()

**ALWAYS use** to isolate operation under test:
- Resets governor limits
- Separates setup from execution
- Forces async operations to complete

```apex
@isTest
static void processLargeDataSet_withManyRecords_completesSuccessfully() {
    // Setup - can use full governor limits
    List<Account> accounts = TestDataFactory.createAccounts(200);
    insert accounts;

    // Execute - fresh governor limits
    Test.startTest();
    AccountService.processAccounts(accounts);
    Test.stopTest();

    // Verify
    List<Account> results = [SELECT Id, Status__c FROM Account];
    System.assertEquals(200, results.size());
}
```

### SeeAllData

**AVOID** `@isTest(SeeAllData=true)` unless absolutely necessary:
- Real data can change anytime
- Causes unpredictable test failures
- Use test data factories instead

**Only acceptable for:**
- Testing with standard objects that can't be created in tests
- Specific integration scenarios

### Mocking

**Use mocking libraries** (e.g., fflib-apex-mocks):

```apex
@isTest
static void getAccounts_withRepositoryError_throwsException() {
    // Mock the dependency
    AccountRepository mockRepo = (AccountRepository)
        Mock.newInstance(AccountRepository.class);

    Mock.when(mockRepo.findActiveAccounts())
        .thenThrow(new QueryException('Database error'));

    AccountService service = new AccountService(mockRepo);

    Test.startTest();
    try {
        service.getActiveAccounts();
        System.assert(false, 'Should have thrown exception');
    } catch (QueryException e) {
        System.assertEquals('Database error', e.getMessage());
    }
    Test.stopTest();
}
```

**Mocking Requirements:**
- Public or global zero-argument constructor
- OR `@testVisible` protected constructor

## Exception Handling

**NEVER ignore exceptions:**

```apex
// ❌ WRONG - Silent failure
try {
    processRecords();
} catch (Exception e) {
    // Empty catch block!
}

// ✅ CORRECT - Log and handle
try {
    processRecords();
} catch (DmlException e) {
    System.debug(LoggingLevel.ERROR, 'Failed to process records: ' + e.getMessage());
    throw new ProcessingException('Unable to complete processing', e);
} catch (Exception e) {
    System.debug(LoggingLevel.ERROR, 'Unexpected error: ' + e.getMessage());
    notifyAdministrator(e);
}
```

## Apex-Specific Best Practices

### Properties

```apex
// Read-only property
public Integer RecordCount { get; }

// Read-write property
public String AccountName { get; set; }

// Write-only property
public String Password { set; }

// Custom getter/setter
private String internalValue;
public String CustomProperty {
    get {
        return internalValue?.toUpperCase();
    }
    set {
        internalValue = value?.trim();
    }
}
```

### Bulkification

**ALWAYS write bulk-safe code:**

```apex
// ✅ CORRECT - Bulk safe
public void updateAccountRatings(List<Account> accounts) {
    List<Account> toUpdate = new List<Account>();

    for (Account acc : accounts) {
        if (acc.AnnualRevenue > 1000000) {
            acc.Rating = 'Hot';
            toUpdate.add(acc);
        }
    }

    if (!toUpdate.isEmpty()) {
        update toUpdate;  // Single DML
    }
}

// ❌ WRONG - Not bulk safe
public void updateAccountRating(Account acc) {
    if (acc.AnnualRevenue > 1000000) {
        acc.Rating = 'Hot';
        update acc;  // DML in loop = governor limit hit
    }
}
```

### Governor Limits

**Key Limits to Monitor:**
- SOQL queries: 100 (synchronous), 200 (async)
- DML statements: 150
- Records per DML: 10,000
- Heap size: 6 MB (synchronous), 12 MB (async)
- CPU time: 10,000 ms (synchronous), 60,000 ms (async)

**Best Practices:**
- Bulkify all operations
- Use collections for DML
- Query outside loops
- Use SOQL for loops for large datasets

```apex
// ✅ CORRECT - Query outside loop
Map<Id, Account> accountMap = new Map<Id, Account>(
    [SELECT Id, Name FROM Account WHERE Id IN :accountIds]
);

for (Contact con : contacts) {
    Account acc = accountMap.get(con.AccountId);
    // process
}

// ❌ WRONG - Query in loop
for (Contact con : contacts) {
    Account acc = [SELECT Id, Name FROM Account WHERE Id = :con.AccountId];
    // Hits SOQL limit with 101 contacts!
}
```

## Quick Reference

### Common Patterns

**Service Class:**
```apex
public with sharing class AccountService {
    private AccountRepository repository;

    public AccountService() {
        this.repository = new AccountRepository();
    }

    public List<Account> getActiveAccounts() {
        return repository.findActiveAccounts();
    }
}
```

**Repository Pattern:**
```apex
public class AccountRepository {
    public List<Account> findActiveAccounts() {
        return [
            SELECT Id, Name, Industry
            FROM Account
            WHERE IsActive__c = true
            ORDER BY Name
        ];
    }
}
```

**Trigger Handler:**
```apex
public class AccountTriggerHandler extends TriggerHandler {
    public override void beforeInsert() {
        for (Account acc : (List<Account>) Trigger.new) {
            validateAccount(acc);
        }
    }

    private void validateAccount(Account acc) {
        // validation logic
    }
}
```

## Key Reminders

1. **Follow K&R brace style** - Opening brace on same line
2. **120 character line limit** - Break at higher syntactic levels
3. **4 spaces for indentation** - NO tabs
4. **One variable per declaration** - Declare close to first use
5. **UpperCamelCase for classes** - Nouns or noun phrases
6. **lowerCamelCase for methods** - Verbs or verb phrases
7. **CONSTANT_CASE for constants** - Static final deeply immutable
8. **Document all public APIs** - Use ApexDoc with at-clauses
9. **Write bulk-safe code** - Handle collections, not single records
10. **Test everything** - Use descriptive test names, Test.startTest/stopTest
11. **Never ignore exceptions** - Log and handle appropriately
12. **SOQL reserved words uppercase** - SELECT, FROM, WHERE, etc.

## References

This skill is based on:
- NimbleUser Apex Style Guide (https://nimbleuser.github.io/apex-style-guide/)
- Google Java Style Guide (adapted for Apex)
- Salesforce Apex Developer Guide
- Apex Best Practices

---

**Created for**: Claude Code
**Purpose**: Ensure consistent, maintainable Apex development following industry standards
**Style Guide**: NimbleUser Apex Style Guide
