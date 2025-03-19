-- Add up migration script here
DROP TABLE IF EXISTS "session";
DROP TABLE IF EXISTS "command";

CREATE TABLE IF NOT EXISTS "session" (
    id INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    uuid UUID UNIQUE NOT NULL,
    cooling_id VARCHAR(250) UNIQUE NOT NULL,
    heating_id VARCHAR(250) UNIQUE NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT now(),
    updated_at TIMESTAMP NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS "command" (
    id INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    uuid UUID UNIQUE NOT NULL,
    session_id INTEGER UNIQUE NOT NULL,
    command_type VARCHAR(250) CHECK (command_type IN ('StartFermentation', 'StopFermentation', 'IncreaseTemperature', 'DecreaseTemperature')) NOT NULL,
    holding_duration INTEGER NOT NULL DEFAULT 0,
    fermentation_step_id INTEGER NOT NULL,
    value NUMERIC(2,1) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT now(),
    updated_at TIMESTAMP NOT NULL DEFAULT now(),
  CONSTRAINT fk_session
      FOREIGN KEY (session_id)
      REFERENCES "session" (id)
      ON DELETE CASCADE
);
