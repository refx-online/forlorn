package main

import (
	_ "github.com/go-sql-driver/mysql"
	"github.com/jmoiron/sqlx"

	"encoding/json"
	"fmt"
	"log"
	"regexp"
	"strings"
)

type Old struct {
	ID   int    `db:"id"`
	File string `db:"file"`
	Name string `db:"name"`
	Desc string `db:"desc"`
	Cond string `db:"cond"`
}

type New struct {
	ID   int             `db:"id"`
	File string          `db:"file"`
	Name string          `db:"name"`
	Desc string          `db:"desc"`
	Cond json.RawMessage `db:"cond"`
}

// change this
var SQL_HOST = "localhost"
var SQL_USER = "root"
var SQL_PASSWORD = ""
var SQL_PORT = "3306"
var SQL_DATABASE = "gulag"

func main() {
	dsn := fmt.Sprintf("%s:%s@(%s:%s)/%s", SQL_USER, SQL_PASSWORD, SQL_HOST, SQL_PORT, SQL_DATABASE)
	db, err := sqlx.Connect("mysql", dsn)
	if err != nil {
		log.Fatalln(err)
	}

	var old []Old
	err = db.Select(&old, "select * from achievements")
	if err != nil {
		log.Fatalln(err)
	}

	for _, ach := range old {
		cond := convertCondition(ach.Cond)

		nr := New{
			ID:   ach.ID,
			File: ach.File,
			Name: ach.Name,
			Desc: ach.Desc,
			Cond: cond,
		}

		_, err := db.NamedExec(
			// gee i wonder who named the table...
			"insert into achievements2 (id, file, name, `desc`, cond) "+
				"values (:id, :file, :name, :desc, :cond)",
			nr,
		)

		if err != nil {
			log.Fatalf("insert fail for %d: %v", ach.ID, err)
		}

		fmt.Printf("migrated achievement %d (%s)\n", ach.ID, ach.Name)
	}

	fmt.Println("migration complete")
}

func convertCondition(src string) json.RawMessage {
	s := strings.TrimSpace(src)
	s = strings.TrimPrefix(s, "(")
	s = strings.TrimSuffix(s, ")")

	parts := splitAnd(s)

	var conditions []map[string]interface{}

	for _, p := range parts {
		p = strings.TrimSpace(p)
		p = strings.TrimPrefix(p, "(")
		p = strings.TrimSuffix(p, ")")

		if strings.Contains(p, "&") && !strings.Contains(p, "==") {
			cond := parseBitConditionNoEquals(p)
			if cond != nil {
				conditions = append(conditions, cond)
				continue
			}
		}

		if strings.Contains(p, "&") && strings.Contains(p, "==") {
			cond := parseBitCondition(p)
			if cond != nil {
				conditions = append(conditions, cond)
				continue
			}
		}

		if strings.Contains(p, "<=") && containsPlainLessThan(p) {
			cond := parseRangeCondition(p)
			if cond != nil {
				conditions = append(conditions, cond)
				continue
			}
		}

		if strings.Contains(p, "<=") && !containsPlainLessThan(p) {
			cond := parseGreaterEqualCondition(p)
			if cond != nil {
				conditions = append(conditions, cond)
				continue
			}
		}

		if strings.Contains(p, "==") {
			cond := parseCompareCondition(p)
			if cond != nil {
				conditions = append(conditions, cond)
				continue
			}
		}

		if strings.HasPrefix(p, "score.") {
			field := strings.TrimPrefix(p, "score.")
			conditions = append(conditions, map[string]interface{}{
				"type":  "compare",
				"stat":  field,
				"op":    "==",
				"value": float64(1),
			})
			continue
		}
	}

	obj := map[string]interface{}{
		"type":       "and",
		"conditions": conditions,
	}

	out, _ := json.Marshal(obj)
	return out
}

func splitAnd(s string) []string {
	return regexp.MustCompile(`\s+and\s+`).Split(s, -1)
}

func containsPlainLessThan(s string) bool {
	c := strings.ReplaceAll(s, "<=", "")
	return strings.Contains(c, "<")
}

func parseBitCondition(src string) map[string]interface{} {
	// score.mods & 1 == 0
	r := regexp.MustCompile(`score\.(\w+)\s*&\s*(\d+)\s*==\s*(\d+)`)
	m := r.FindStringSubmatch(src)
	if len(m) == 4 {
		return map[string]interface{}{
			"type":   "bit_eq",
			"stat":   m[1],
			"mask":   atoi(m[2]),
			"equals": atoi(m[3]),
		}
	}
	return nil
}

func parseBitConditionNoEquals(src string) map[string]interface{} {
	// score.mods & 8
	r := regexp.MustCompile(`score\.(\w+)\s*&\s*(\d+)`)
	m := r.FindStringSubmatch(src)
	if len(m) == 3 {
		return map[string]interface{}{
			"type": "bit_ne",
			"stat": m[1],
			"mask": atoi(m[2]),
		}
	}
	return nil
}

func parseRangeCondition(src string) map[string]interface{} {
	// 1 <= score.sr < 2
	r := regexp.MustCompile(`(\d+(?:\.\d+)?)\s*<=\s*score\.(\w+)\s*<\s*(\d+(?:\.\d+)?)`)
	m := r.FindStringSubmatch(src)
	if len(m) >= 4 {
		return map[string]interface{}{
			"type": "range",
			"stat": m[2],
			"min":  atof(m[1]),
			"max":  atof(m[3]),
		}
	}
	return nil
}

func parseCompareCondition(src string) map[string]interface{} {
	// mode_vn == 0 or score.mods == 32
	r := regexp.MustCompile(`(?:score\.)?(\w+)\s*==\s*(\d+(?:\.\d+)?)`)
	m := r.FindStringSubmatch(src)
	if len(m) >= 3 {
		return map[string]interface{}{
			"type":  "compare",
			"stat":  m[1],
			"op":    "==",
			"value": atof(m[2]),
		}
	}
	return nil
}

func parseGreaterEqualCondition(src string) map[string]interface{} {
	// 2000 <= score.max_combo
	r := regexp.MustCompile(`(\d+(?:\.\d+)?)\s*<=\s*score\.(\w+)`)
	m := r.FindStringSubmatch(src)
	if len(m) >= 3 {
		return map[string]interface{}{
			"type":  "compare",
			"stat":  m[2],
			"op":    ">=",
			"value": atof(m[1]),
		}
	}
	return nil
}

func atoi(s string) int {
	var v int
	fmt.Sscan(s, &v)
	return v
}

func atof(s string) float64 {
	var v float64
	fmt.Sscan(s, &v)
	return v
}
