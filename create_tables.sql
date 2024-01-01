CREATE TABLE IF NOT EXISTS Seats (
    seat_id INTEGER PRIMARY KEY,
    available BOOLEAN NOT NULL,
    other_info TEXT
);

CREATE TABLE IF NOT EXISTS Users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_name TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE,
    user_role TEXT NOT NULL,
    verified BOOLEAN NOT NULL,
    verification_token TEXT
);

CREATE TABLE IF NOT EXISTS Reservations (
    user_name TEXT NOT NULL,
    seat_id INTEGER NOT NULL,
    start_time TEXT NOT NULL,
    end_time TEXT NOT NULL,
    PRIMARY KEY (user_name, start_time, end_time),
    FOREIGN KEY(user_name) REFERENCES Users(user_name),
    FOREIGN KEY(seat_id) REFERENCES Seats(seat_id)
);

CREATE TABLE IF NOT EXISTS UnavailableTimeSlots (
    start_time TEXT NOT NULL,
    end_time TEXT NOT NULL,
    PRIMARY KEY (start_time, end_time)
);

CREATE TABLE IF NOT EXISTS BlackList (
    user_name TEXT NOT NULL,
    start_time TEXT NOT NULL,
    end_time TEXT NOT NULL,
    PRIMARY KEY (user_name),
    FOREIGN KEY(user_name) REFERENCES Users(user_name)
);