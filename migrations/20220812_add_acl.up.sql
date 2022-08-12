--;;
ALTER TABLE biominer_indexd_file ADD COLUMN acl VARCHAR(255) DEFAULT NULL; -- such as 'admin,default,fudan-pgx'. It means the file is public when acl is NULL, elsewise private.

--;;
COMMENT ON COLUMN biominer_indexd_file.acl IS 'Access control list with strings identifying required authorizations';