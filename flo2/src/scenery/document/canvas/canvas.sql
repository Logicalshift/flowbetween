/***
 **
 ** Flowbetween file format version 5
 **
 ***/

/**
 *  Maps property names to IDs in this document
 **/
CREATE TABLE Properties (
    PropertyId  INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    Name        TEXT    NOT NULL
);

/**
 * Properties that apply to the whole document with int values
 **/
CREATE TABLE DocumentIntProperties (
    PropertyId  INTEGER NOT NULL,
    IntValue    INTEGER NOT NULL,

    PRIMARY KEY (PropertyId)
);

/**
 * Properties that apply to the whole document with float values
 **/
CREATE TABLE DocumentFloatProperties (
    PropertyId  INTEGER NOT NULL,
    FloatValue  FLOAT   NOT NULL,

    PRIMARY KEY (PropertyId)
);

/**
 * Properties that apply to the whole document but are encoded as postcard blob values (serialized `CanvasProperty` values)
 **/
CREATE TABLE DocumentBlobProperties (
    PropertyId  INTEGER NOT NULL,
    BlobValue   BLOB    NOT NULL,

    PRIMARY KEY (PropertyId)
);

/**
 * The layers that make up this document
 **/
CREATE TABLE Layers (
    LayerId     INTEGER     NOT NULL PRIMARY KEY AUTOINCREMENT,
    LayerGuid   CHAR(36)    NOT NULL,
    Idx         INTEGER     NOT NULL
);

/**
 * Integer properties attached to a layer
 **/
CREATE TABLE LayerIntProperties (
    LayerId     INTEGER NOT NULL,
    PropertyId  INTEGER NOT NULL,
    IntValue    INTEGER NOT NULL,

    PRIMARY KEY (LayerId, PropertyId)
);

/**
 * Float properties attached to a layer
 **/
CREATE TABLE LayerFloatProperties (
    LayerId     INTEGER NOT NULL,
    PropertyId  INTEGER NOT NULL,
    FloatValue  FLOAT   NOT NULL,

    PRIMARY KEY (LayerId, PropertyId)
);

/**
 * Blob properties attached to a layer (postcard serialized `CanvasProperty` values)
 **/
CREATE TABLE LayerBlobProperties (
    LayerId     INTEGER NOT NULL,
    PropertyId  INTEGER NOT NULL,
    BlobValue   BLOB    NOT NULL,

    PRIMARY KEY (LayerId, PropertyId)
);

/**
 * The shapes that are in the canvas, as a list
 * 
 * Shape type can be:
 *  0 - bezier path
 *  1 - group
 *  2 - rectangle
 *  3 - ellipse
 *  4 - polygon
 **/
CREATE TABLE Shapes (
    ShapeId     INTEGER     NOT NULL,
    ShapeGuid   CHAR(36)    NOT NULL,
    ShapeType   INTEGER     NOT NULL,

    PRIMARY KEY (ShapeId)
) WITHOUT ROWID;

/**
 * For shapes that are on a layer, this associates them with that layer
 **/
CREATE TABLE ShapeLayers (
    ShapeId     INTEGER NOT NULL,
    LayerId     INTEGER NOT NULL,
    OrderIdx    INTEGER NOT NULL,

    PRIMARY KEY (LayerId, OrderIdx, ShapeId)
) WITHOUT ROWID;

/**
 * For shapes that are part of a group, this associates them with their parent shape
 **/
CREATE TABLE ShapeGroups (
    ShapeId         INTEGER NOT NULL,
    ParentShapeId   INTEGER NOT NULL,
    OrderIdx        INTEGER NOT NULL,

    PRIMARY KEY (ShapeId, OrderIdx, ParentShapeId)
);

/**
 * The points that make up each shape
 **/
CREATE TABLE ShapePoints (
    ShapeId     INTEGER NOT NULL,
    PointIdx    INTEGER NOT NULL,
    X           FLOAT   NOT NULL,
    Y           FLOAT   NOT NULL,

    PRIMARY KEY (ShapeId, PointIdx)
);

/**
 * Integer properties attached to a shape
 **/
CREATE TABLE ShapeIntProperties (
    ShapeId     INTEGER NOT NULL,
    PropertyId  INTEGER NOT NULL,
    IntValue    INTEGER NOT NULL,

    PRIMARY KEY (ShapeId, PropertyId)
);

/**
 * Float properties attached to a shape
 **/
CREATE TABLE ShapeFloatProperties (
    ShapeId     INTEGER NOT NULL,
    PropertyId  INTEGER NOT NULL,
    FloatValue  FLOAT   NOT NULL,

    PRIMARY KEY (ShapeId, PropertyId)
);

/**
 * Blob properties attached to a shape (postcard serialized `CanvasProperty` values)
 **/
CREATE TABLE ShapeBlobProperties (
    ShapeId     INTEGER NOT NULL,
    PropertyId  INTEGER NOT NULL,
    BlobValue   BLOB    NOT NULL,

    PRIMARY KEY (ShapeId, PropertyId)
);

/**
  * The brushes that are defined for this document
  **/
 CREATE TABLE Brushes (
     BrushId     INTEGER    NOT NULL PRIMARY KEY AUTOINCREMENT,
     BrushGuid   CHAR(36)   NOT NULL
 );

/**
 * Integer properties attached to a shape
 **/
CREATE TABLE BrushIntProperties (
    BrushId     INTEGER NOT NULL,
    PropertyId  INTEGER NOT NULL,
    IntValue    INTEGER NOT NULL,

    PRIMARY KEY (BrushId, PropertyId)
);

/**
 * Float properties attached to a shape
 **/
CREATE TABLE BrushFloatProperties (
    BrushId     INTEGER NOT NULL,
    PropertyId  INTEGER NOT NULL,
    FloatValue  FLOAT   NOT NULL,

    PRIMARY KEY (BrushId, PropertyId)
);

/**
 * Blob properties attached to a shape (postcard serialized `CanvasProperty` values)
 **/
CREATE TABLE BrushBlobProperties (
    BrushId     INTEGER NOT NULL,
    PropertyId  INTEGER NOT NULL,
    BlobValue   BLOB    NOT NULL,

    PRIMARY KEY (BrushId, PropertyId)
);
