# mess

## Simple service ~~mess~~mesh written in rust!

## features

## Architecture of Mess

```mermaid
graph
    Client/Server
    Client/Server <--> S1_M
    Client/Server <--> S2_M
    S1_M <--> S2_M

    subgraph Mess
        subgraph Centralized Store
            CS[(SQLite)]
        end

        subgraph Service N
            S1_M[Muck]
            S1_1[Service Instance N]
            S1_2[Service Instance N+1]
            S1_DB[(SQLite)]
            S1_M --> S1_DB
            S1_1 <--> S1_M
            S1_2 <--> S1_M
            S1_DB <-->|Sync| CS
        end

        subgraph Service N+1
            S2_M[Muck]
            S2_1[Service Instance N]
            S2_2[Service Instance N+1]
            S2_DB[(SQLite)]
            S2_M --> S2_DB
            S2_1 <--> S2_M
            S2_2 <--> S2_M
            S2_DB <-->|Sync| CS
        end
    end
```

### Dev Docs

Deps:

- [sqitch](https://sqitch.org/) (v1.4.1)
- [SQLite](https://www.sqlite.org/index.html) (Version 3.45.3)

#### database

To deploy migration and setup db:

    `sqitch deploy db:sqlite:mess.db`

for info on how to migrate [here](https://sqitch.org/docs/manual/sqitchtutorial-sqlite/) is a good sqitch tutorial
