# Apex Common Patterns & Examples

Real-world patterns for effective Apex development following NimbleUser style guide standards.

## Service Layer Pattern

### Basic Service Class

```apex
/**
 * @description Service for managing account-related business logic
 */
public with sharing class AccountService {
    private AccountRepository repository;
    private ValidationService validator;

    public AccountService() {
        this.repository = new AccountRepository();
        this.validator = new ValidationService();
    }

    /**
     * @description Retrieves active accounts for specified industry
     * @param industry The industry to filter accounts
     * @param minRevenue Minimum annual revenue threshold
     * @return List of matching accounts
     */
    public List<Account> getActiveAccounts(String industry, Decimal minRevenue) {
        validator.validateIndustry(industry);
        validator.validateRevenue(minRevenue);

        return repository.findActiveAccountsByIndustry(industry, minRevenue);
    }

    /**
     * @description Creates new accounts with validation
     * @param accountsToCreate List of accounts to create
     * @return List of created accounts with IDs
     */
    public List<Account> createAccounts(List<Account> accountsToCreate) {
        validateAccountList(accountsToCreate);

        for (Account acc : accountsToCreate) {
            enrichAccountData(acc);
        }

        insert accountsToCreate;
        return accountsToCreate;
    }

    private void validateAccountList(List<Account> accounts) {
        if (accounts == null || accounts.isEmpty()) {
            throw new ServiceException('Account list cannot be empty');
        }
    }

    private void enrichAccountData(Account acc) {
        if (String.isBlank(acc.AccountSource)) {
            acc.AccountSource = 'Web';
        }
        acc.CreatedFromService__c = true;
    }
}
```

## Repository Pattern

### Data Access Layer

```apex
/**
 * @description Repository for account data access operations
 */
public class AccountRepository {
    private static final Integer DEFAULT_LIMIT = 200;

    /**
     * @description Finds active accounts by industry and revenue
     * @param industry Industry filter
     * @param minRevenue Minimum revenue threshold
     * @return List of matching accounts
     */
    public List<Account> findActiveAccountsByIndustry(
        String industry,
        Decimal minRevenue
    ) {
        return [
            SELECT Id, Name, Industry, AnnualRevenue, Rating,
                (SELECT Id, FirstName, LastName, Email FROM Contacts)
            FROM Account
            WHERE IsActive__c = true
                AND Industry = :industry
                AND AnnualRevenue >= :minRevenue
            ORDER BY AnnualRevenue DESC
            LIMIT :DEFAULT_LIMIT
        ];
    }

    /**
     * @description Finds account by ID with related data
     * @param accountId Account record ID
     * @return Account with related records
     */
    public Account findByIdWithRelated(Id accountId) {
        List<Account> accounts = [
            SELECT Id, Name, Industry, AnnualRevenue,
                (SELECT Id, Name FROM Opportunities WHERE IsClosed = false),
                (SELECT Id, FirstName, LastName FROM Contacts)
            FROM Account
            WHERE Id = :accountId
            LIMIT 1
        ];

        if (accounts.isEmpty()) {
            throw new RepositoryException('Account not found: ' + accountId);
        }

        return accounts[0];
    }

    /**
     * @description Finds accounts by IDs using a map for bulk operations
     * @param accountIds Set of account IDs to retrieve
     * @return Map of account ID to Account record
     */
    public Map<Id, Account> findByIds(Set<Id> accountIds) {
        return new Map<Id, Account>([
            SELECT Id, Name, Industry, AnnualRevenue, Rating
            FROM Account
            WHERE Id IN :accountIds
        ]);
    }

    /**
     * @description Searches accounts by name pattern
     * @param searchTerm Name search term
     * @return List of matching accounts
     */
    public List<Account> searchByName(String searchTerm) {
        String searchPattern = '%' + searchTerm + '%';

        return [
            SELECT Id, Name, Industry
            FROM Account
            WHERE Name LIKE :searchPattern
            ORDER BY Name
            LIMIT 50
        ];
    }
}
```

## Trigger Handler Pattern

### Base Trigger Handler

```apex
/**
 * @description Base class for trigger handlers providing common functionality
 */
public virtual class TriggerHandler {
    protected Boolean isBefore;
    protected Boolean isAfter;
    protected Boolean isInsert;
    protected Boolean isUpdate;
    protected Boolean isDelete;
    protected Boolean isUndelete;

    public TriggerHandler() {
        this.isBefore = Trigger.isBefore;
        this.isAfter = Trigger.isAfter;
        this.isInsert = Trigger.isInsert;
        this.isUpdate = Trigger.isUpdate;
        this.isDelete = Trigger.isDelete;
        this.isUndelete = Trigger.isUndelete;
    }

    /**
     * @description Main entry point for trigger execution
     */
    public void run() {
        if (isBefore) {
            if (isInsert) {
                beforeInsert();
            } else if (isUpdate) {
                beforeUpdate();
            } else if (isDelete) {
                beforeDelete();
            }
        } else if (isAfter) {
            if (isInsert) {
                afterInsert();
            } else if (isUpdate) {
                afterUpdate();
            } else if (isDelete) {
                afterDelete();
            } else if (isUndelete) {
                afterUndelete();
            }
        }
    }

    // Override these methods in subclasses
    protected virtual void beforeInsert() {}
    protected virtual void beforeUpdate() {}
    protected virtual void beforeDelete() {}
    protected virtual void afterInsert() {}
    protected virtual void afterUpdate() {}
    protected virtual void afterDelete() {}
    protected virtual void afterUndelete() {}
}
```

### Concrete Trigger Handler

```apex
/**
 * @description Handler for Account trigger operations
 */
public class AccountTriggerHandler extends TriggerHandler {
    private List<Account> newAccounts;
    private List<Account> oldAccounts;
    private Map<Id, Account> newAccountMap;
    private Map<Id, Account> oldAccountMap;

    public AccountTriggerHandler() {
        super();
        this.newAccounts = (List<Account>) Trigger.new;
        this.oldAccounts = (List<Account>) Trigger.old;
        this.newAccountMap = (Map<Id, Account>) Trigger.newMap;
        this.oldAccountMap = (Map<Id, Account>) Trigger.oldMap;
    }

    protected override void beforeInsert() {
        setDefaultValues();
        validateAccounts();
    }

    protected override void beforeUpdate() {
        validateAccounts();
        preventInvalidUpdates();
    }

    protected override void afterInsert() {
        createDefaultOpportunities();
        sendNotifications();
    }

    protected override void afterUpdate() {
        handleRatingChanges();
    }

    private void setDefaultValues() {
        for (Account acc : newAccounts) {
            if (String.isBlank(acc.AccountSource)) {
                acc.AccountSource = 'Other';
            }
            if (acc.Rating == null) {
                acc.Rating = 'Warm';
            }
        }
    }

    private void validateAccounts() {
        for (Account acc : newAccounts) {
            if (String.isBlank(acc.Name)) {
                acc.addError('Account Name is required');
            }
            if (acc.AnnualRevenue != null && acc.AnnualRevenue < 0) {
                acc.AnnualRevenue.addError('Annual Revenue cannot be negative');
            }
        }
    }

    private void preventInvalidUpdates() {
        for (Account acc : newAccounts) {
            Account oldAccount = oldAccountMap.get(acc.Id);

            if (oldAccount.IsLocked__c && acc.IsLocked__c) {
                acc.addError('Cannot modify locked account');
            }
        }
    }

    private void createDefaultOpportunities() {
        List<Opportunity> oppsToCreate = new List<Opportunity>();

        for (Account acc : newAccounts) {
            if (acc.CreateDefaultOpportunity__c) {
                oppsToCreate.add(new Opportunity(
                    Name = acc.Name + ' - Default Opportunity',
                    AccountId = acc.Id,
                    StageName = 'Prospecting',
                    CloseDate = Date.today().addDays(30)
                ));
            }
        }

        if (!oppsToCreate.isEmpty()) {
            insert oppsToCreate;
        }
    }

    private void sendNotifications() {
        Set<Id> ownerIds = new Set<Id>();

        for (Account acc : newAccounts) {
            if (acc.AnnualRevenue > 10000000) {
                ownerIds.add(acc.OwnerId);
            }
        }

        if (!ownerIds.isEmpty()) {
            NotificationService.notifyHighValueAccounts(ownerIds);
        }
    }

    private void handleRatingChanges() {
        List<Account> ratingChanged = new List<Account>();

        for (Account acc : newAccounts) {
            Account oldAccount = oldAccountMap.get(acc.Id);

            if (acc.Rating != oldAccount.Rating) {
                ratingChanged.add(acc);
            }
        }

        if (!ratingChanged.isEmpty()) {
            RatingChangeService.processRatingChanges(ratingChanged);
        }
    }
}
```

### Trigger Definition

```apex
/**
 * @description Trigger for Account object
 */
trigger AccountTrigger on Account (
    before insert, before update, before delete,
    after insert, after update, after delete, after undelete
) {
    new AccountTriggerHandler().run();
}
```

## Bulk Processing Patterns

### Processing Large Data Sets

```apex
/**
 * @description Batch class for processing large account datasets
 */
public class AccountUpdateBatch implements Database.Batchable<SObject> {
    private String industry;
    private String newRating;

    public AccountUpdateBatch(String industry, String newRating) {
        this.industry = industry;
        this.newRating = newRating;
    }

    public Database.QueryLocator start(Database.BatchableContext bc) {
        return Database.getQueryLocator([
            SELECT Id, Name, Industry, Rating
            FROM Account
            WHERE Industry = :industry
        ]);
    }

    public void execute(Database.BatchableContext bc, List<Account> scope) {
        for (Account acc : scope) {
            acc.Rating = newRating;
            acc.LastProcessedDate__c = System.now();
        }

        update scope;
    }

    public void finish(Database.BatchableContext bc) {
        AsyncApexJob job = [
            SELECT Id, Status, NumberOfErrors, JobItemsProcessed, TotalJobItems
            FROM AsyncApexJob
            WHERE Id = :bc.getJobId()
        ];

        System.debug('Batch job completed: ' + job.Status);
        System.debug('Records processed: ' + job.TotalJobItems);
    }
}
```

### Queueable for Chain Processing

```apex
/**
 * @description Queueable job for account enrichment
 */
public class AccountEnrichmentJob implements Queueable, Database.AllowsCallouts {
    private List<Id> accountIds;
    private Integer batchNumber;

    public AccountEnrichmentJob(List<Id> accountIds, Integer batchNumber) {
        this.accountIds = accountIds;
        this.batchNumber = batchNumber;
    }

    public void execute(QueueableContext context) {
        List<Account> accounts = [
            SELECT Id, Name, Website
            FROM Account
            WHERE Id IN :accountIds
        ];

        for (Account acc : accounts) {
            enrichFromExternalService(acc);
        }

        update accounts;

        // Chain next batch if more records exist
        List<Id> nextBatch = getNextBatch();
        if (!nextBatch.isEmpty()) {
            System.enqueueJob(new AccountEnrichmentJob(nextBatch, batchNumber + 1));
        }
    }

    private void enrichFromExternalService(Account acc) {
        if (String.isNotBlank(acc.Website)) {
            // Callout to external service
            HttpRequest req = new HttpRequest();
            req.setEndpoint('https://api.example.com/enrich');
            req.setMethod('GET');
            req.setHeader('Content-Type', 'application/json');

            Http http = new Http();
            HttpResponse res = http.send(req);

            if (res.getStatusCode() == 200) {
                // Process response
                Map<String, Object> data =
                    (Map<String, Object>) JSON.deserializeUntyped(res.getBody());
                acc.EnrichedData__c = JSON.serialize(data);
            }
        }
    }

    private List<Id> getNextBatch() {
        // Implementation to get next batch of IDs
        return new List<Id>();
    }
}
```

## SOQL Patterns

### Dynamic SOQL

```apex
/**
 * @description Builds dynamic SOQL queries safely
 */
public class DynamicQueryBuilder {
    private String objectName;
    private List<String> fields;
    private List<String> conditions;
    private String orderBy;
    private Integer limitCount;

    public DynamicQueryBuilder(String objectName) {
        this.objectName = objectName;
        this.fields = new List<String>();
        this.conditions = new List<String>();
    }

    public DynamicQueryBuilder selectFields(List<String> fields) {
        this.fields.addAll(fields);
        return this;
    }

    public DynamicQueryBuilder whereCondition(String field, String operator, Object value) {
        String condition = field + ' ' + operator + ' :' + field.replace('.', '_');
        conditions.add(condition);
        return this;
    }

    public DynamicQueryBuilder orderBy(String field, String direction) {
        this.orderBy = field + ' ' + direction;
        return this;
    }

    public DynamicQueryBuilder setLimit(Integer limitCount) {
        this.limitCount = limitCount;
        return this;
    }

    public String build() {
        String query = 'SELECT ' + String.join(fields, ', ');
        query += ' FROM ' + objectName;

        if (!conditions.isEmpty()) {
            query += ' WHERE ' + String.join(conditions, ' AND ');
        }

        if (String.isNotBlank(orderBy)) {
            query += ' ORDER BY ' + orderBy;
        }

        if (limitCount != null) {
            query += ' LIMIT ' + limitCount;
        }

        return query;
    }
}
```

### SOQL for Large Datasets

```apex
/**
 * @description Processes large datasets efficiently
 */
public class LargeDataProcessor {
    private static final Integer BATCH_SIZE = 200;

    /**
     * @description Processes all accounts using SOQL for loop
     */
    public void processAllAccounts() {
        Integer totalProcessed = 0;

        // SOQL for loop automatically chunks into 200-record batches
        for (List<Account> accounts : [
            SELECT Id, Name, Industry, AnnualRevenue
            FROM Account
            WHERE IsActive__c = true
        ]) {
            processAccountBatch(accounts);
            totalProcessed += accounts.size();
        }

        System.debug('Total accounts processed: ' + totalProcessed);
    }

    private void processAccountBatch(List<Account> accounts) {
        for (Account acc : accounts) {
            // Process each account
            calculateAccountScore(acc);
        }

        update accounts;
    }

    private void calculateAccountScore(Account acc) {
        Decimal score = 0;

        if (acc.AnnualRevenue != null) {
            score += acc.AnnualRevenue / 1000000;
        }

        if (acc.Industry == 'Technology') {
            score += 10;
        }

        acc.AccountScore__c = score;
    }
}
```

## Test Data Factory Pattern

```apex
/**
 * @description Factory for creating test data
 */
@isTest
public class TestDataFactory {
    private static Integer accountCounter = 0;
    private static Integer contactCounter = 0;

    /**
     * @description Creates a single account with default values
     * @param accountName Name for the account
     * @return Account record (not inserted)
     */
    public static Account createAccount(String accountName) {
        accountCounter++;

        return new Account(
            Name = accountName,
            Industry = 'Technology',
            AnnualRevenue = 1000000,
            Rating = 'Hot',
            Phone = '555-0' + String.valueOf(accountCounter).leftPad(3, '0')
        );
    }

    /**
     * @description Creates multiple accounts
     * @param count Number of accounts to create
     * @return List of account records (not inserted)
     */
    public static List<Account> createAccounts(Integer count) {
        List<Account> accounts = new List<Account>();

        for (Integer i = 0; i < count; i++) {
            accounts.add(createAccount('Test Account ' + (accountCounter + 1)));
        }

        return accounts;
    }

    /**
     * @description Creates contacts for an account
     * @param accountId Parent account ID
     * @param count Number of contacts to create
     * @return List of contact records (not inserted)
     */
    public static List<Contact> createContacts(Id accountId, Integer count) {
        List<Contact> contacts = new List<Contact>();

        for (Integer i = 0; i < count; i++) {
            contactCounter++;
            contacts.add(new Contact(
                AccountId = accountId,
                FirstName = 'Test',
                LastName = 'Contact ' + contactCounter,
                Email = 'test' + contactCounter + '@example.com',
                Phone = '555-1' + String.valueOf(contactCounter).leftPad(3, '0')
            ));
        }

        return contacts;
    }

    /**
     * @description Creates complete account hierarchy
     * @return Map with account, contacts, and opportunities
     */
    public static Map<String, Object> createAccountHierarchy() {
        Account acc = createAccount('Test Corporation');
        insert acc;

        List<Contact> contacts = createContacts(acc.Id, 3);
        insert contacts;

        List<Opportunity> opps = new List<Opportunity>{
            new Opportunity(
                Name = acc.Name + ' - Opportunity 1',
                AccountId = acc.Id,
                StageName = 'Prospecting',
                CloseDate = Date.today().addDays(30),
                Amount = 50000
            ),
            new Opportunity(
                Name = acc.Name + ' - Opportunity 2',
                AccountId = acc.Id,
                StageName = 'Qualification',
                CloseDate = Date.today().addDays(60),
                Amount = 100000
            )
        };
        insert opps;

        return new Map<String, Object>{
            'account' => acc,
            'contacts' => contacts,
            'opportunities' => opps
        };
    }
}
```

## Error Handling Pattern

```apex
/**
 * @description Custom exception classes
 */
public class ServiceException extends Exception {}
public class RepositoryException extends Exception {}
public class ValidationException extends Exception {}

/**
 * @description Centralized error handling
 */
public class ErrorHandler {
    /**
     * @description Logs error and sends notification if needed
     * @param ex Exception to handle
     * @param context Context where error occurred
     */
    public static void handleError(Exception ex, String context) {
        // Log the error
        System.debug(LoggingLevel.ERROR,
            'Error in ' + context + ': ' + ex.getMessage() + '\n' + ex.getStackTraceString()
        );

        // Create error log record
        Error_Log__c log = new Error_Log__c(
            Context__c = context,
            Message__c = ex.getMessage(),
            StackTrace__c = ex.getStackTraceString(),
            Type__c = ex.getTypeName(),
            Timestamp__c = System.now()
        );

        try {
            insert log;
        } catch (DmlException dmlEx) {
            System.debug(LoggingLevel.ERROR, 'Failed to log error: ' + dmlEx.getMessage());
        }

        // Send notification for critical errors
        if (isCriticalError(ex)) {
            sendErrorNotification(ex, context);
        }
    }

    private static Boolean isCriticalError(Exception ex) {
        return ex instanceof System.LimitException
            || ex instanceof System.AssertException;
    }

    private static void sendErrorNotification(Exception ex, String context) {
        // Send email to administrators
        Messaging.SingleEmailMessage mail = new Messaging.SingleEmailMessage();
        mail.setToAddresses(new List<String>{ 'admin@example.com' });
        mail.setSubject('Critical Error: ' + context);
        mail.setPlainTextBody(
            'An error occurred in ' + context + '\n\n' +
            'Message: ' + ex.getMessage() + '\n\n' +
            'Stack Trace:\n' + ex.getStackTraceString()
        );

        try {
            Messaging.sendEmail(new List<Messaging.SingleEmailMessage>{ mail });
        } catch (Exception emailEx) {
            System.debug(LoggingLevel.ERROR,
                'Failed to send error notification: ' + emailEx.getMessage()
            );
        }
    }
}
```

## Key Takeaways

1. **Separation of Concerns**: Use service, repository, and handler layers
2. **Bulkification**: Always handle collections, never single records
3. **SOQL Efficiency**: Query outside loops, use SOQL for loops for large datasets
4. **Trigger Patterns**: Use handler classes with clear method structure
5. **Test Data**: Use factories for consistent, reusable test data
6. **Error Handling**: Log, notify, and handle gracefully
7. **Async Processing**: Use Batch, Queueable, or Future for heavy operations
8. **Governor Limits**: Design with limits in mind from the start
