CREATE TABLE IF NOT EXISTS biominer_indexd_file (
  guid VARCHAR(64) PRIMARY KEY, -- The file's global unique identifier
  filename VARCHAR(255) NOT NULL, -- An optional name for the file that will be searchable through Indexd's API
  size BIGINT NOT NULL, -- Size of the file in bytes
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP, -- When the file was created
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP, -- When the file was last updated
  status VARCHAR(16) NOT NULL DEFAULT 'pending', -- 'pending', 'processing', 'validated', 'failed'
  rev VARCHAR(8) NOT NULL DEFAULT 'no_rev', -- The current revision (for avoiding conflicts)
  baseid VARCHAR(64) NOT NULL, -- The base identifier linking logically similar GUIDs
  uploader VARCHAR(64) DEFAULT 'biominer-admin', -- The user who uploaded the file
  version INTEGER NOT NULL DEFAULT 1 -- The version of the file schema
);

CREATE TABLE IF NOT EXISTS biominer_indexd_url (
  id BIGSERIAL PRIMARY KEY, -- The URL's unique identifier
  url VARCHAR(255) NOT NULL UNIQUE, -- The URL to the file, gsa://path/to/file, node://path/to/file, oss://bucket/path/to/file, s3://bucket/path/to/file, etc.
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP, -- When the URL was created
  status VARCHAR(16) NOT NULL DEFAULT 'pending', -- 'pending', 'processing', 'validated', 'failed'
  uploader VARCHAR(64) NOT NULL DEFAULT 'biominer-admin', -- The user who uploaded the file

  file VARCHAR(64) NOT NULL, -- The file's global unique identifier
  FOREIGN KEY (file) REFERENCES biominer_indexd_file(guid)
);

CREATE TABLE IF NOT EXISTS biominer_indexd_hash (
  id BIGSERIAL PRIMARY KEY, -- The hash's unique identifier
  hash VARCHAR(128) NOT NULL UNIQUE, -- The hash of the file, max 128 characters, md5, sha1, sha256, sha512, blake2b, etc.
  hash_type VARCHAR(16) NOT NULL DEFAULT 'md5', -- The hash type, md5, sha1, sha256, sha512, blake2b, etc.

  file VARCHAR(64) NOT NULL, -- The file's global unique identifier
  FOREIGN KEY (file) REFERENCES biominer_indexd_file(guid)
);

CREATE TABLE IF NOT EXISTS biominer_indexd_alias (
  id BIGSERIAL PRIMARY KEY, -- The alias's unique identifier
  name VARCHAR(255) NOT NULL UNIQUE, -- The alias of the file, max 255 characters, doi://10.1234/5678, etc.

  file VARCHAR(64) NOT NULL, -- The file's global unique identifier
  FOREIGN KEY (file) REFERENCES biominer_indexd_file(guid)
);