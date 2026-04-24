# Project Plan

by Logan DeNezza (denezza.4)

## Intro

Hollow Knight - Charms, Notches, Upgrades, Shops, and more

Hollow Knight is an indie videogame developed by Team Cherry in 2017. It sold
15 million copies, making it one of the most successful indie games of all time.

This project will map transactions between useful game items, such as charms,
notches, mask shards, and spells. These interact with multiple locations,
sublocations, shops, vendors, side-quests and more.

## ER Schema

There is no `ER-model.png` checked into this repository; the diagram below is the
reference layout for the tables in **DB Schema** (charms and mask shards each
have parallel location, requirement, and vendor rows; vendors also have a row
in **Vendor Locations**).

```mermaid
erDiagram
    CHARMS_LOCATIONS {
        int id PK
        string location_name
    }
    CHARMS_REQUIREMENTS {
        int id PK
        string condition_text
    }
    CHARMS_VENDOR {
        int id PK
        string vendor_name
        int cost
    }
    MASK_SHARDS_LOCATIONS {
        int id PK
        string location_name
    }
    MASK_SHARDS_REQUIREMENTS {
        int id PK
        string condition_text
    }
    MASK_SHARDS_VENDOR {
        int id PK
        string vendor_name
        int cost
    }
    VENDOR_LOCATIONS {
        string vendor_name PK
        string location_name
    }
    CHARMS_VENDOR }o--|| VENDOR_LOCATIONS : "vendor_name"
    MASK_SHARDS_VENDOR }o--|| VENDOR_LOCATIONS : "vendor_name"
```

## DB Schema
an asterisk indicates primary key

Charms Locations
|id\*|location name|
--------------------

Charms Requirements
|id\*|condition text|
---------------------

Charms Vendor
|id\*|vendor name|cost|
-----------------------

Mask Shards Locations
|id\*|location name|
--------------------

Mask Shards Requirements
|id\*|condition text|
---------------------

Mask Shards Vendor
|id\*|vendor name|cost|
-----------------------

Vendor Locations
|vendor name\*|location name|
-----------------------------

## DBMS

I will use SQLite, as it is a free DBMS.

## Programming Language

I will be using Rust as my programming language of choice.

