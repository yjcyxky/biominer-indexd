INSERT INTO "public"."biominer_indexd_file" ("guid", "filename", "size", "created_at", "updated_at", "status", "rev", "baseid", "uploader", "version") VALUES ('abcd', 'dfdfdf', 50, '2022-04-06 08:02:37.539042', '2022-04-06 08:02:37.539042', 'failed', 'no_rev', 'rest', 'test', 1);

INSERT INTO "public"."biominer_indexd_alias" ("name", "file") VALUES ('test', 'abcd');
INSERT INTO "public"."biominer_indexd_alias" ("name", "file") VALUES ('test1', 'abcd');

INSERT INTO "public"."biominer_indexd_hash" ("hash", "hash_type", "file") VALUES ('testset', 'md5', 'abcd');
INSERT INTO "public"."biominer_indexd_hash" ("hash", "hash_type", "file") VALUES ('md5test', 'sha256', 'abcd');

INSERT INTO "public"."biominer_indexd_url" ("url", "created_at", "status", "uploader", "file") VALUES ('oss://test', '2022-04-06 08:03:17.42371', 'failed', 'test', 'abcd');
