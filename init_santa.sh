#!/bin/bash

~/.surrealdb/surreal sql --namespace test --database test --username root --password root --pretty --endpoint tikv://127.0.0.1:2379 <<EOF
CREATE user:santa SET name = 'Santa', pass = crypto::argon2::generate('hohoho'), roles = ['Admin'];
CREATE user:elf1 SET name = 'Oscar', pass = crypto::argon2::generate('123'), roles = ['Default', 'Admin'];
CREATE user:elf2 SET name = 'Henriette', pass = crypto::argon2::generate('kuchen'), roles = ['Default', 'Admin'];
CREATE user:elf3 SET name = 'Kevin', pass = crypto::argon2::generate('321'), roles = ['Default', 'Admin'];
EOF

