# nlql

Talk to your database in plain english.

> ⚠️ Work in progress

## Usage

```bash
# set your api key
export ANTHROPIC_API_KEY="sk-..."

# ask your database a question
nlql query "show all users" --db "postgres:///mydb"

# just see the sql without running it
nlql query "count orders by status" --db "postgres:///mydb" --dry-run

# raw json output
nlql query "top 5 orders by amount" --db "postgres:///mydb" --output raw

# show schema
nlql schema --db "postgres:///mydb"

# run as http server
nlql serve --db "postgres:///mydb" --port 3000
```

## Development

```bash
nix develop     # enter dev shell
pg_up           # start local postgres
pg_down         # stop postgres
```

## License

MIT
