FROM postgres:15
RUN mkdir -p /certs

COPY docker/postgres/postgresql.conf /etc/postgresql/postgresql.conf
COPY docker/postgres/pg_hba.conf /etc/postgresql/pg_hba.conf

COPY certs/server/server.crt /certs/
COPY certs/server/server.key /certs/
COPY certs/ca.crt /certs/


RUN chmod 600 /certs/*.key && \
  chmod 644 /certs/*.crt

