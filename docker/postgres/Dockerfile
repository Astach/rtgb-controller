FROM postgres:17

COPY docker/postgres/postgresql.conf /etc/postgresql/postgresql.conf
COPY docker/postgres/pg_hba.conf /etc/postgresql/pg_hba.conf


COPY certs/server/server.crt /etc/postgresql/server.crt
COPY certs/server/server.key /etc/postgresql/server.key
COPY certs/ca.crt /etc/postgresql/ca.crt

RUN chown postgres:postgres /etc/postgresql/server.crt /etc/postgresql/server.key /etc/postgresql/ca.crt && \
  chmod 600 /etc/postgresql/*.key && \
  chmod 644 /etc/postgresql/*.crt

COPY docker/postgres/initdb.sh /docker-entrypoint-initdb.d/
RUN chmod +x /docker-entrypoint-initdb.d/initdb.sh
