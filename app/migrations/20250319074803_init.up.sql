-- Add up migration script here
DROP TABLE IF EXISTS "session";
DROP TABLE IF EXISTS "command";

CREATE TABLE IF NOT EXISTS "session" (
    id INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    uuid UUID UNIQUE NOT NULL,
    cooling_id VARCHAR(250) NOT NULL,
    heating_id VARCHAR(250) NOT NULL,
    created_at TIMESTAMP(6) NOT NULL DEFAULT now(),
    updated_at TIMESTAMP(6) NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS "command" (
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
      REFERENCES "session" (id)
      ON DELETE CASCADE
);
