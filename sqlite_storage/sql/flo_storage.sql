/***
 **
 ** FlowBetween File format version 4
 **
 **   V4 of the file format moves the bulk of the work of data representation into the animation and its serialization
 **   format, which greatly simplifies the content of the database.
 **
 ***/

/**
 * Represents the global properties for the animation
 */
CREATE TABLE AnimationProperties (
    PropertyId INTEGER NOT NULL PRIMARY KEY,
    Value TEXT NOT NULL
) WITHOUT ROWID;

/** 
 * A log of all the edits the user has performed to the animation
 */
CREATE TABLE EditLog (
    EditId INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    Edit TEXT NOT NULL
);

/**
 * An element definition
 */
CREATE TABLE Elements (
    ElementId INTEGER NOT NULL PRIMARY KEY,
    Element TEXT NOT NULL
) WITHOUT ROWID;

/**
 * A layer definition
 */
CREATE TABLE Layers (
    LayerId INTEGER NOT NULL PRIMARY KEY,
    Layer TEXT NOT NULL
) WITHOUT ROWID;

/**
 * A keyframe definition
 */
CREATE TABLE Keyframe (
    LayerId INTEGER NOT NULL,
    TimeMicroseconds INTEGER NOT NULL,

    PRIMARY KEY (LayerId, TimeMicroseconds)
) WITHOUT ROWID;

/**
 * Where an element is attached to a layer
 */
CREATE TABLE ElementKeyframeAttachment (
    ElementId INTEGER NOT NULL,
    LayerId INTEGER NOT NULL,
    TimeMicroseconds INTEGER NOT NULL,

    PRIMARY KEY (LayerId, TimeMicroseconds, ElementId)
) WITHOUT ROWID;

/* Index to look up where an element is attached */
CREATE INDEX Idx_ElementAttachments ON ElementKeyframeAttachment (ElementId, LayerId, TimeMicroseconds);

/**
 * Cached values for a particular layer
 */
CREATE TABLE LayerCache (
    LayerId INTEGER NOT NULL,
    TimeMicroseconds INTEGER NOT NULL,
    CacheType TEXT NOT NULL,
    Cache TEXT NOT NULL,

    PRIMARY KEY (LayerId, CacheType, TimeMicroseconds)
) WITHOUT ROWID;
