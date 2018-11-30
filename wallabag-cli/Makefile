
DB_FILE="db.sqlite3"

.PHONY: clean

migrate: ${DB_FILE}

clean:
	rm -f ${DB_FILE}

${DB_FILE}:
	diesel migration run --database-url ${DB_FILE}
