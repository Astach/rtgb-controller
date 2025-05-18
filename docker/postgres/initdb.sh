#!/bin/bash
psql -v ON_ERROR_STOP=1 <<-EOSQL

	  CREATE ROLE ${TEST_USER} WITH SUPERUSER LOGIN PASSWORD '${TEST_USER_PASSWORD}'; --used for running tests.
		CREATE DATABASE ${POSTGRES_RTGB_DB}; 

		  \c ${POSTGRES_RTGB_DB}

		CREATE USER ${POSTGRES_SERVICE_USER};

		  CREATE TABLE IF NOT EXISTS ${POSTGRES_SESSION_TABLE_NAME} (
		      id INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
		      uuid UUID UNIQUE NOT NULL,
		      cooling_id VARCHAR(250) NOT NULL,
		      heating_id VARCHAR(250)  NOT NULL,
	          created_at TIMESTAMP(6) NOT NULL DEFAULT now(),
	          updated_at TIMESTAMP(6) NOT NULL DEFAULT now()
		  );

		  CREATE TABLE IF NOT EXISTS ${POSTGRES_COMMAND_TABLE_NAME} (
	        id INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	        uuid UUID UNIQUE NOT NULL,
	        fermentation_step_id INTEGER NOT NULL,
	        status VARCHAR(250) CHECK (status IN ('Planned', 'Running', 'Executed')), 
	          status_date TIMESTAMP(6),
	        value NUMERIC(3,1) NOT NULL,
	          value_reached_at TIMESTAMP(6),
	        value_holding_duration INTEGER NOT NULL, -- for how long to maintain the temperature after the value (target temp ) as been reached.
	          created_at TIMESTAMP(6) NOT NULL DEFAULT now(),
	          updated_at TIMESTAMP(6) NOT NULL DEFAULT now(),
	        session_id INTEGER NOT NULL,
		    CONSTRAINT fk_session
		        FOREIGN KEY (session_id)
		        REFERENCES ${POSTGRES_SESSION_TABLE_NAME} (id)
		        ON DELETE CASCADE
		  );

		GRANT SELECT,INSERT,UPDATE,DELETE ON TABLE ${POSTGRES_COMMAND_TABLE_NAME} TO ${POSTGRES_SERVICE_USER};
		GRANT SELECT,INSERT,UPDATE,DELETE ON TABLE ${POSTGRES_SESSION_TABLE_NAME} TO ${POSTGRES_SERVICE_USER};
EOSQL
