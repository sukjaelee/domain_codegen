CREATE TABLE todos (
    id CHAR(36) NOT NULL PRIMARY KEY,             -- UUID as CHAR(36)
    user_id CHAR(36) NOT NULL,                     -- Foreign key to users table
    title VARCHAR(255) NOT NULL,                   -- Title of the todo
    description TEXT,                              -- Optional detailed description
    status VARCHAR(32) NOT NULL DEFAULT 'pending', -- Status like pending/completed
    due_date TIMESTAMP NULL,                       -- Optional due date
    created_by CHAR(36) NOT NULL,                  -- Who created
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP, -- Creation timestamp
    modified_by CHAR(36) NOT NULL,                 -- Who last modified
    modified_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP -- Modification timestamp
);