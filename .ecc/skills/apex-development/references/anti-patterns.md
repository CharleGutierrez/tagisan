# Apex Anti-Patterns & Code Smells

Critical "Do's and Don'ts" for Apex development to avoid common pitfalls and maintain code quality.

## Governor Limit Anti-Patterns

### ❌ NEVER: SOQL in Loops

**CRITICAL**: This is the #1 cause of governor limit exceptions.

**WRONG:**
```apex
// ❌ WRONG - SOQL inside loop
public void updateContactAccounts(List<Contact> contacts) {
    for (Contact con : contacts) {
        // Query executes for EACH contact - hits 100 query limit!
        Account acc = [SELECT Id, Name FROM Account WHERE Id = :con.AccountId];
        acc.Status__c = 'Updated';
        update acc;  // DML in loop too!
    }
}
```

**CORRECT:**
```apex
// ✅ CORRECT - Query once, process in bulk
public void updateContactAccounts(List<Contact> contacts) {
    Set<Id> accountIds = new Set<Id>();
    for (Contact con : contacts) {
        accountIds.add(con.AccountId);
    }

    Map<Id, Account> accountMap = new Map<Id, Account>(
        [SELECT Id, Name FROM Account WHERE Id IN :accountIds]
    );

    List<Account> accountsToUpdate = new List<Account>();
    for (Contact con : contacts) {
        Account acc = accountMap.get(con.AccountId);
        if (acc != null) {
            acc.Status__c = 'Updated';
            accountsToUpdate.add(acc);
        }
    }

    if (!accountsToUpdate.isEmpty()) {
        update accountsToUpdate;
    }
}
```

### ❌ NEVER: DML in Loops

**WRONG:**
```apex
// ❌ WRONG - DML inside loop
public void createOpportunities(List<Account> accounts) {
    for (Account acc : accounts) {
        Opportunity opp = new Opportunity(
            Name = acc.Name + ' - Opportunity',
            AccountId = acc.Id,
            StageName = 'Prospecting',
            CloseDate = Date.today().addDays(30)
        );
        insert opp;  // Hits 150 DML limit with 151 accounts!
    }
}
```

**CORRECT:**
```apex
// ✅ CORRECT - Collect and insert once
public void createOpportunities(List<Account> accounts) {
    List<Opportunity> oppsToInsert = new List<Opportunity>();

    for (Account acc : accounts) {
        oppsToInsert.add(new Opportunity(
            Name = acc.Name + ' - Opportunity',
            AccountId = acc.Id,
            StageName = 'Prospecting',
            CloseDate = Date.today().addDays(30)
        ));
    }

    if (!oppsToInsert.isEmpty()) {
        insert oppsToInsert;
    }
}
```

### ❌ NEVER: Query All Records Without LIMIT

**WRONG:**
```apex
// ❌ WRONG - No limit, could return millions
public List<Account> getAllAccounts() {
    return [SELECT Id, Name FROM Account];  // Heap size exception waiting to happen!
}
```

**CORRECT:**
```apex
// ✅ CORRECT - Use appropriate limits or batch processing
public List<Account> getActiveAccounts() {
    return [
        SELECT Id, Name
        FROM Account
        WHERE IsActive__c = true
        ORDER BY Name
        LIMIT 200
    ];
}

// ✅ Or use batch for large datasets
public class ProcessAllAccountsBatch implements Database.Batchable<SObject> {
    public Database.QueryLocator start(Database.BatchableContext bc) {
        return Database.getQueryLocator([SELECT Id, Name FROM Account]);
    }

    public void execute(Database.BatchableContext bc, List<Account> scope) {
        // Process batch
    }

    public void finish(Database.BatchableContext bc) {}
}
```

### ❌ NEVER: Hardcoded IDs

**WRONG:**
```apex
// ❌ WRONG - IDs differ between orgs (sandbox, production, etc.)
public static final Id SYSTEM_ADMIN_PROFILE = '00e1234567890ABCD';
public static final Id DEFAULT_RECORD_TYPE = '0121234567890ABCD';

Account acc = new Account(
    Name = 'Test',
    RecordTypeId = '0121234567890ABCD'  // Won't work in other orgs!
);
```

**CORRECT:**
```apex
// ✅ CORRECT - Query IDs dynamically
public class ProfileHelper {
    private static Map<String, Id> profileCache;

    public static Id getProfileId(String profileName) {
        if (profileCache == null) {
            profileCache = new Map<String, Id>();
            for (Profile p : [SELECT Id, Name FROM Profile]) {
                profileCache.put(p.Name, p.Id);
            }
        }
        return profileCache.get(profileName);
    }
}

// ✅ CORRECT - Use developer name
RecordType rt = [
    SELECT Id
    FROM RecordType
    WHERE SObjectType = 'Account'
        AND DeveloperName = 'Business_Account'
    LIMIT 1
];

Account acc = new Account(
    Name = 'Test',
    RecordTypeId = rt.Id
);
```

## Non-Bulkified Code Anti-Patterns

### ❌ NEVER: Single-Record Methods in Triggers

**WRONG:**
```apex
// ❌ WRONG - Trigger handler expects single record
trigger AccountTrigger on Account (before insert) {
    for (Account acc : Trigger.new) {
        AccountHandler.processAccount(acc);  // Called once per record!
    }
}

public class AccountHandler {
    public static void processAccount(Account acc) {
        List<Contact> contacts = [
            SELECT Id FROM Contact WHERE AccountId = :acc.Id
        ];  // SOQL in loop!
        // process contacts
    }
}
```

**CORRECT:**
```apex
// ✅ CORRECT - Handler expects collection
trigger AccountTrigger on Account (before insert) {
    AccountHandler.processAccounts(Trigger.new);
}

public class AccountHandler {
    public static void processAccounts(List<Account> accounts) {
        Set<Id> accountIds = new Set<Id>();
        for (Account acc : accounts) {
            accountIds.add(acc.Id);
        }

        Map<Id, List<Contact>> contactsByAccount = new Map<Id, List<Contact>>();
        for (Contact con : [SELECT Id, AccountId FROM Contact WHERE AccountId IN :accountIds]) {
            if (!contactsByAccount.containsKey(con.AccountId)) {
                contactsByAccount.put(con.AccountId, new List<Contact>());
            }
            contactsByAccount.get(con.AccountId).add(con);
        }

        // Process all accounts together
        for (Account acc : accounts) {
            List<Contact> contacts = contactsByAccount.get(acc.Id);
            // process contacts
        }
    }
}
```

### ❌ DON'T: Assume Single Record in Trigger

**WRONG:**
```apex
// ❌ WRONG - Assumes only one record
trigger AccountTrigger on Account (after insert) {
    Account acc = Trigger.new[0];  // What if multiple records?
    // process only first account
}
```

**CORRECT:**
```apex
// ✅ CORRECT - Handle all records
trigger AccountTrigger on Account (after insert) {
    for (Account acc : Trigger.new) {
        // process each account
    }
    // Or better: call handler with full list
    AccountTriggerHandler.handleAfterInsert(Trigger.new);
}
```

## Trigger Anti-Patterns

### ❌ NEVER: Business Logic in Triggers

**WRONG:**
```apex
// ❌ WRONG - All logic in trigger (hard to test, maintain, reuse)
trigger AccountTrigger on Account (before insert, after insert, before update) {
    if (Trigger.isBefore && Trigger.isInsert) {
        for (Account acc : Trigger.new) {
            if (String.isBlank(acc.AccountSource)) {
                acc.AccountSource = 'Other';
            }
            if (acc.AnnualRevenue != null && acc.AnnualRevenue < 0) {
                acc.addError('Revenue cannot be negative');
            }
        }
    }

    if (Trigger.isAfter && Trigger.isInsert) {
        List<Opportunity> opps = new List<Opportunity>();
        for (Account acc : Trigger.new) {
            opps.add(new Opportunity(
                Name = acc.Name + ' - Opp',
                AccountId = acc.Id,
                StageName = 'Prospecting',
                CloseDate = Date.today().addDays(30)
            ));
        }
        insert opps;
    }

    // More logic...
}
```

**CORRECT:**
```apex
// ✅ CORRECT - Trigger delegates to handler
trigger AccountTrigger on Account (
    before insert, before update,
    after insert, after update
) {
    new AccountTriggerHandler().run();
}

// Handler contains all logic
public class AccountTriggerHandler extends TriggerHandler {
    protected override void beforeInsert() {
        setDefaultValues();
        validateAccounts();
    }

    protected override void afterInsert() {
        createDefaultOpportunities();
    }

    // Testable, reusable methods
    private void setDefaultValues() { /* logic */ }
    private void validateAccounts() { /* logic */ }
    private void createDefaultOpportunities() { /* logic */ }
}
```

### ❌ NEVER: Multiple Triggers per Object

**WRONG:**
```apex
// ❌ WRONG - Multiple triggers (execution order undefined!)
trigger AccountTrigger1 on Account (before insert) {
    // logic 1
}

trigger AccountTrigger2 on Account (before insert) {
    // logic 2 - which runs first? Nobody knows!
}

trigger AccountAfterTrigger on Account (after insert) {
    // logic 3
}
```

**CORRECT:**
```apex
// ✅ CORRECT - One trigger per object
trigger AccountTrigger on Account (
    before insert, before update, before delete,
    after insert, after update, after delete, after undelete
) {
    new AccountTriggerHandler().run();
}

// Handler manages all contexts
public class AccountTriggerHandler extends TriggerHandler {
    protected override void beforeInsert() { /* logic 1 */ }
    protected override void beforeUpdate() { /* logic 2 */ }
    protected override void afterInsert() { /* logic 3 */ }
    // etc.
}
```

### ❌ DON'T: Recursive Triggers Without Guard

**WRONG:**
```apex
// ❌ WRONG - No recursion guard
trigger AccountTrigger on Account (after update) {
    List<Account> toUpdate = new List<Account>();
    for (Account acc : Trigger.new) {
        Account relatedAcc = new Account(Id = acc.ParentId, Status__c = 'Updated');
        toUpdate.add(relatedAcc);
    }
    update toUpdate;  // Triggers AccountTrigger again = infinite loop!
}
```

**CORRECT:**
```apex
// ✅ CORRECT - Use recursion guard
public class TriggerHelper {
    private static Set<String> executedTriggers = new Set<String>();

    public static Boolean hasRun(String triggerName) {
        if (executedTriggers.contains(triggerName)) {
            return true;
        }
        executedTriggers.add(triggerName);
        return false;
    }
}

trigger AccountTrigger on Account (after update) {
    if (TriggerHelper.hasRun('AccountTrigger_afterUpdate')) {
        return;
    }

    List<Account> toUpdate = new List<Account>();
    for (Account acc : Trigger.new) {
        Account relatedAcc = new Account(Id = acc.ParentId, Status__c = 'Updated');
        toUpdate.add(relatedAcc);
    }

    if (!toUpdate.isEmpty()) {
        update toUpdate;
    }
}
```

## SOQL Anti-Patterns

### ❌ DON'T: Query for Fields You Don't Use

**WRONG:**
```apex
// ❌ WRONG - Querying all fields when only need name
public String getAccountName(Id accountId) {
    Account acc = [
        SELECT Id, Name, Industry, AnnualRevenue, Phone, Website,
            BillingStreet, BillingCity, BillingState, Rating, Type
        FROM Account
        WHERE Id = :accountId
    ];
    return acc.Name;  // Only using Name!
}
```

**CORRECT:**
```apex
// ✅ CORRECT - Query only needed fields
public String getAccountName(Id accountId) {
    Account acc = [
        SELECT Name
        FROM Account
        WHERE Id = :accountId
        LIMIT 1
    ];
    return acc.Name;
}
```

### ❌ DON'T: Use SOQL When You Have the Data

**WRONG:**
```apex
// ❌ WRONG - Re-querying data you already have
public void processAccounts(List<Account> accounts) {
    for (Account acc : accounts) {
        // Re-querying the same account!
        Account freshAcc = [SELECT Id, Name FROM Account WHERE Id = :acc.Id];
        System.debug(freshAcc.Name);
    }
}
```

**CORRECT:**
```apex
// ✅ CORRECT - Use the data you have
public void processAccounts(List<Account> accounts) {
    for (Account acc : accounts) {
        System.debug(acc.Name);  // Use the account from the list
    }
}

// ✅ Only re-query if you need fresh data or additional fields
public void processAccountsWithFreshData(Set<Id> accountIds) {
    List<Account> accounts = [
        SELECT Id, Name, Industry, (SELECT Id FROM Contacts)
        FROM Account
        WHERE Id IN :accountIds
    ];

    for (Account acc : accounts) {
        System.debug(acc.Name + ' has ' + acc.Contacts.size() + ' contacts');
    }
}
```

### ❌ DON'T: Use Count() When You Need Records

**WRONG:**
```apex
// ❌ WRONG - Query twice
public void processIfAccountsExist(String industry) {
    Integer count = [SELECT COUNT() FROM Account WHERE Industry = :industry];

    if (count > 0) {
        List<Account> accounts = [
            SELECT Id, Name FROM Account WHERE Industry = :industry
        ];  // Querying same data again!
        processAccounts(accounts);
    }
}
```

**CORRECT:**
```apex
// ✅ CORRECT - Query once and check
public void processIfAccountsExist(String industry) {
    List<Account> accounts = [
        SELECT Id, Name
        FROM Account
        WHERE Industry = :industry
    ];

    if (!accounts.isEmpty()) {
        processAccounts(accounts);
    }
}
```

## Test Anti-Patterns

### ❌ NEVER: Use SeeAllData=true

**WRONG:**
```apex
// ❌ WRONG - Depends on org data (unstable, can change anytime)
@isTest(SeeAllData=true)
private class AccountServiceTest {
    @isTest
    static void testGetAccounts() {
        List<Account> accounts = AccountService.getAccounts();
        System.assertNotEquals(null, accounts);
        // Test can fail if data changes!
    }
}
```

**CORRECT:**
```apex
// ✅ CORRECT - Create test data
@isTest
private class AccountServiceTest {
    @testSetup
    static void setup() {
        List<Account> testAccounts = new List<Account>();
        for (Integer i = 0; i < 10; i++) {
            testAccounts.add(new Account(
                Name = 'Test Account ' + i,
                Industry = 'Technology'
            ));
        }
        insert testAccounts;
    }

    @isTest
    static void getAccounts_withTestData_returnsAccounts() {
        Test.startTest();
        List<Account> accounts = AccountService.getAccounts();
        Test.stopTest();

        System.assertEquals(10, accounts.size(), 'Should return 10 test accounts');
    }
}
```

### ❌ DON'T: Assert Without Messages

**WRONG:**
```apex
// ❌ WRONG - No assertion messages (hard to debug when fails)
@isTest
static void testAccountCreation() {
    Account acc = new Account(Name = 'Test');
    insert acc;

    Account result = [SELECT Id, Name FROM Account WHERE Id = :acc.Id];
    System.assertNotEquals(null, result);  // Why did this fail?
    System.assertEquals('Test', result.Name);  // Which assertion failed?
}
```

**CORRECT:**
```apex
// ✅ CORRECT - Clear assertion messages
@isTest
static void createAccount_withValidData_createsSuccessfully() {
    // Setup
    Account acc = new Account(Name = 'Test Corp');

    // Execute
    Test.startTest();
    insert acc;
    Test.stopTest();

    // Verify
    Account result = [SELECT Id, Name FROM Account WHERE Id = :acc.Id];
    System.assertNotEquals(null, result, 'Account should be created');
    System.assertEquals('Test Corp', result.Name, 'Account name should match');
    System.assertNotEquals(null, result.Id, 'Account should have an ID after insert');
}
```

### ❌ DON'T: Skip Test.startTest() and Test.stopTest()

**WRONG:**
```apex
// ❌ WRONG - No Test.startTest/stopTest (governor limits mix with setup)
@isTest
static void testBatchProcessing() {
    List<Account> accounts = createTestAccounts(200);  // Uses queries
    insert accounts;  // Uses DML

    // This batch might hit limits because setup already used some!
    Database.executeBatch(new AccountBatch(), 200);
}
```

**CORRECT:**
```apex
// ✅ CORRECT - Separate setup and test governor limits
@isTest
static void processBatch_withManyAccounts_completesSuccessfully() {
    // Setup - has its own governor limits
    List<Account> accounts = createTestAccounts(200);
    insert accounts;

    // Execute - gets FRESH governor limits
    Test.startTest();
    Database.executeBatch(new AccountBatch(), 200);
    Test.stopTest();  // Forces batch to complete

    // Verify
    List<Account> results = [SELECT Id, Status__c FROM Account];
    System.assertEquals(200, results.size());
}
```

### ❌ DON'T: Test Implementation Details

**WRONG:**
```apex
// ❌ WRONG - Testing private methods directly (brittle, breaks on refactoring)
@isTest
static void testCalculateScore() {
    AccountService service = new AccountService();
    // Can't test private methods directly - and shouldn't!
}
```

**CORRECT:**
```apex
// ✅ CORRECT - Test public API and verify outcomes
@isTest
static void processAccount_withHighRevenue_setsHighScore() {
    // Setup
    Account acc = new Account(
        Name = 'Test Corp',
        AnnualRevenue = 5000000
    );
    insert acc;

    // Execute
    Test.startTest();
    AccountService.processAccount(acc.Id);
    Test.stopTest();

    // Verify the outcome (score was calculated correctly)
    Account result = [SELECT AccountScore__c FROM Account WHERE Id = :acc.Id];
    System.assert(result.AccountScore__c > 0, 'High revenue should produce high score');
}
```

## Exception Handling Anti-Patterns

### ❌ NEVER: Empty Catch Blocks

**WRONG:**
```apex
// ❌ WRONG - Swallowing exceptions
public void processRecords(List<Account> accounts) {
    try {
        update accounts;
    } catch (DmlException e) {
        // Empty catch - error is silently ignored!
    }
}
```

**CORRECT:**
```apex
// ✅ CORRECT - Log and handle appropriately
public void processRecords(List<Account> accounts) {
    try {
        update accounts;
    } catch (DmlException e) {
        System.debug(LoggingLevel.ERROR,
            'Failed to update accounts: ' + e.getMessage()
        );
        ErrorHandler.logError(e, 'AccountService.processRecords');
        throw new ServiceException('Unable to process accounts', e);
    }
}
```

### ❌ DON'T: Catch Generic Exception for Specific Errors

**WRONG:**
```apex
// ❌ WRONG - Too broad, might catch unexpected errors
public void processData() {
    try {
        performDmlOperation();
    } catch (Exception e) {  // Catches EVERYTHING
        // What if it's a LimitException? NullPointerException?
        System.debug('Some error occurred');
    }
}
```

**CORRECT:**
```apex
// ✅ CORRECT - Catch specific exceptions
public void processData() {
    try {
        performDmlOperation();
    } catch (DmlException e) {
        System.debug('DML operation failed: ' + e.getMessage());
        handleDmlError(e);
    } catch (QueryException e) {
        System.debug('Query failed: ' + e.getMessage());
        handleQueryError(e);
    } catch (Exception e) {
        // Unexpected error - log and escalate
        System.debug('Unexpected error: ' + e.getMessage());
        throw e;
    }
}
```

## Code Organization Anti-Patterns

### ❌ DON'T: God Classes

**WRONG:**
```apex
// ❌ WRONG - One class doing everything (3000+ lines)
public class AccountManager {
    // Account creation
    public void createAccount() { /* 200 lines */ }

    // Account validation
    public void validateAccount() { /* 150 lines */ }

    // Contact management
    public void createContact() { /* 200 lines */ }

    // Opportunity handling
    public void createOpportunity() { /* 300 lines */ }

    // Email notifications
    public void sendEmail() { /* 100 lines */ }

    // Reporting
    public void generateReport() { /* 400 lines */ }

    // Integration
    public void callExternalApi() { /* 250 lines */ }

    // ... 20 more methods ...
}
```

**CORRECT:**
```apex
// ✅ CORRECT - Separate concerns into focused classes
public class AccountService {
    private AccountRepository repository;
    private AccountValidator validator;
    private NotificationService notifier;

    public void createAccount(Account acc) {
        validator.validate(acc);
        repository.insert(acc);
        notifier.notifyAccountCreated(acc);
    }
}

public class AccountValidator {
    public void validate(Account acc) { /* focused validation */ }
}

public class AccountRepository {
    public void insert(Account acc) { /* data access */ }
}

public class NotificationService {
    public void notifyAccountCreated(Account acc) { /* notifications */ }
}
```

### ❌ DON'T: Utility Classes with State

**WRONG:**
```apex
// ❌ WRONG - Utility class with instance state
public class StringUtils {
    private String currentValue;  // State in utility class!

    public String format(String input) {
        this.currentValue = input;
        return currentValue.trim();
    }
}
```

**CORRECT:**
```apex
// ✅ CORRECT - Stateless utility class
public class StringUtils {
    // Private constructor prevents instantiation
    private StringUtils() {}

    public static String format(String input) {
        return input != null ? input.trim() : '';
    }

    public static Boolean isBlank(String input) {
        return input == null || input.trim().length() == 0;
    }
}
```

## Security Anti-Patterns

### ❌ NEVER: String Concatenation in SOQL (Injection Risk)

**WRONG:**
```apex
// ❌ WRONG - SOQL injection vulnerability!
public List<Account> searchAccounts(String searchTerm) {
    String query = 'SELECT Id, Name FROM Account WHERE Name LIKE \'%' +
        searchTerm + '%\'';  // searchTerm could be: %' OR IsDeleted = false OR Name LIKE '%
    return Database.query(query);
}
```

**CORRECT:**
```apex
// ✅ CORRECT - Use bind variables
public List<Account> searchAccounts(String searchTerm) {
    String searchPattern = '%' + searchTerm + '%';
    return [
        SELECT Id, Name
        FROM Account
        WHERE Name LIKE :searchPattern
    ];
}

// ✅ Also correct - Escape special characters for dynamic SOQL
public List<Account> searchAccountsDynamic(String searchTerm) {
    String escaped = String.escapeSingleQuotes(searchTerm);
    String query = 'SELECT Id, Name FROM Account WHERE Name LIKE \'%' +
        escaped + '%\'';
    return Database.query(query);
}
```

### ❌ DON'T: Ignore Sharing Rules When Not Needed

**WRONG:**
```apex
// ❌ WRONG - without sharing allows access to all records
public without sharing class AccountService {
    // Should this really bypass sharing rules?
    public List<Account> getAllAccounts() {
        return [SELECT Id, Name FROM Account];
    }
}
```

**CORRECT:**
```apex
// ✅ CORRECT - Use 'with sharing' by default
public with sharing class AccountService {
    public List<Account> getAllAccounts() {
        return [SELECT Id, Name FROM Account];  // Respects user's access
    }
}

// ✅ Only use 'without sharing' when there's a clear business need
public without sharing class SystemAccountService {
    // Explicitly documented why sharing is bypassed
    /**
     * @description Retrieves all accounts for system-level operations
     * NOTE: Bypasses sharing rules - use only for admin operations
     */
    public List<Account> getAllAccountsForAdmin() {
        return [SELECT Id, Name FROM Account];
    }
}
```

## Key Takeaways

1. **Bulkify Everything**: Always handle collections, never single records
2. **SOQL/DML Outside Loops**: Query and DML once, process many
3. **One Trigger Per Object**: Use handler pattern for organization
4. **Test with Real Data**: Use test data factories, avoid SeeAllData
5. **Handle Exceptions**: Never use empty catch blocks
6. **Respect Limits**: Design with governor limits from the start
7. **Security First**: Use bind variables, respect sharing rules
8. **Separate Concerns**: Avoid god classes, use focused services
9. **Test Behavior**: Test outcomes, not implementation details
10. **Guard Recursion**: Use static flags to prevent infinite loops
