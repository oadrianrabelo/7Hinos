import sqlite3

db = r"C:/Program Files (x86)/Louvor JA/config/database.db"
con = sqlite3.connect(db)
cur = con.cursor()

cur.execute("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
tables = [r[0] for r in cur.fetchall()]
print("TABLES:", tables)

for t in tables:
    cur.execute(f"PRAGMA table_info([{t}])")
    cols = [(c[1], c[2]) for c in cur.fetchall()]
    cur.execute(f"SELECT COUNT(*) FROM [{t}]")
    n = cur.fetchone()[0]
    print(f"\n{t} ({n} rows):")
    for name, typ in cols:
        print(f"  {name}  {typ}")

con.close()
