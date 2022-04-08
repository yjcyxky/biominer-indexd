INSERT INTO "public"."biominer_indexd_file" ("guid", "filename", "size", "created_at", "updated_at", "status", "rev", "baseid", "uploader", "version") VALUES ('abcd', 'dfdfdf', 50, EXTRACT(EPOCH from now()) * 1000, EXTRACT(EPOCH from now()) * 1000, 'failed', 'no_rev', 'rest', 'test', 1);

INSERT INTO "public"."biominer_indexd_alias" ("name", "file") VALUES ('test', 'abcd');
INSERT INTO "public"."biominer_indexd_alias" ("name", "file") VALUES ('test1', 'abcd');

INSERT INTO "public"."biominer_indexd_hash" ("hash", "hash_type", "file") VALUES ('testset', 'md5', 'abcd');
INSERT INTO "public"."biominer_indexd_hash" ("hash", "hash_type", "file") VALUES ('md5test', 'sha256', 'abcd');

INSERT INTO "public"."biominer_indexd_url" ("url", "created_at", "status", "uploader", "file") VALUES ('oss://test', EXTRACT(EPOCH from now()) * 1000, 'failed', 'test', 'abcd');
