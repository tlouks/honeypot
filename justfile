set dotenv-load

alias r := run

run:
    cargo r --release

postgres:
    docker run -d \
        --name some-postgres \
        -e POSTGRES_PASSWORD=dev \
        -e POSTGRES_USER=tyler \
        -e POSTGRES_DB=honeypot \
        -e PGDATA=/var/lib/postgresql/data/pgdata \
        -v /home/tyler/dev/honeypot2/data:/var/lib/postgresql/data \
        -p 5432:5432 \
        postgres