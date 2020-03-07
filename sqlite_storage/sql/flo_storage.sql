/***
 **
 ** FlowBetween File format version 4
 **
 ***/

/**
 * Represents the global properties for the animation
 */
CREATE TABLE AnimationProperties (
    PropertyId INTEGER NOT NULL AUTOINCREMENT PRIMARY KEY,
    Value TEXT NOT NULL
);

/** 
 * A log of all the edits the user has performed to the animation
 */
CREATE TABLE EditLog (
    EditId INTEGER NOT NULL AUTOINCREMENT PRIMARY KEY,
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
