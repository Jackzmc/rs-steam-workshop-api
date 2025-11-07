# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

# 0.7.0
### Changes
- Finished implementation of search_items
- Made some fields public
- Fixed all tests

## 0.6.0
### Changes
- Added `subscribe(&self, publishedfileid: &str)`
- Added `unsubscribe(&self, publishedfileid: &str)`
- Removed accidental hardcoded key

## 0.5.0

### Updated
- Rewrote search_items to allow more query paramaters
- search_items now returns a cursor, allowing you to continue searching
- Made tests use `STEAM_API_KEY` environmental variable for auth-required methods
