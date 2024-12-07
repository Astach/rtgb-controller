FROM nats:2.10-alpine3.20
RUN mkdir -p /certs

COPY docker/nats/server.conf /etc/nats/server.conf
COPY certs/server/server.crt /certs/
COPY certs/server/server.key /certs/
COPY certs/ca.crt /certs/


RUN chmod 600 /certs/*.key && \
  chmod 644 /certs/*.crt
