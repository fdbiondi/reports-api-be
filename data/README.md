## Create tables

### Enter to sqlite on console executing `sqlite3`

#

### To create the reports and nonces table execute the following queries

#### create reports table

`CREATE TABLE reports (uuid NVARCHAR(36) UNIQUE NOT NULL, signature NVARCHAR(132) PRIMARY KEY NOT NULL, description TEXT NOT NULL, title NVARCHAR(50) NOT NULL, state NVARCHAR(12) NOT NULL);`

#### create nonces table

`CREATE TABLE nonces (uuid NVARCHAR(36) UNIQUE NOT NULL, signature NVARCHAR(132) PRIMARY KEY NOT NULL, nonce INTEGER NOT NULL);`

#### select report by signature

`SELECT * FROM reports where signature = '{signature}'`

#### select nonce by signature

`SELECT * FROM nonces where signature = '{signature}'`

#### create new report

`INSERT INTO reports (uuid, signature, description, title, state) VALUES (?,?,?,?,?)`

#### create new nonce

`INSERT INTO nonces (uuid, signature, nonce) VALUES(?,?)`

#### update nonce

`UPDATE nonces SET nonce = ? WHERE signature = ?`
