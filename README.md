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

I will use MySQL, as it is a free DBMS.

## Programming Language

I will be using Rust as my programming language of choice.

