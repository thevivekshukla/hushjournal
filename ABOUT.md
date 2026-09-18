e2ejournal is end-to-end encrypted journal web app. It encrypts user content using AES-256-GCM-SIV on the client side, server side only stores the encrypted content and can't ever see the plain content.

User can log in using Google oauth2, they can create multiple "Journal", each journal can have one or more "Notebook", each notebook will have entries. An entry is title and content (both encrypted on the client). By default when they create new entry its title will have prefilled title as today's date such as "7 Sep 2026".

Journal:
- user_id
- key_salt
- encrypted_dek

Notebook
- journal_id
- name (encrypted)

Entry:
- notebook_id
- title (encrypted)
- content (encrypted)
