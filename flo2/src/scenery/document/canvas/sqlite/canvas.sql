/***
 **
 ** Flowbetween file format version 5
 **
 ***/

PRAGMA foreign_keys = ON;

/**
 *  Maps property names to IDs in this document
 **/
CREATE TABLE Properties (
    PropertyId  INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    Name        TEXT    NOT NULL
);

/**
 *  Maps shape type names to IDs in this document
 **/
CREATE TABLE ShapeTypes (
    ShapeTypeId INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    Name        TEXT    NOT NULL
);

/**
 * Properties that apply to the whole document with int values
 **/
CREATE TABLE DocumentIntProperties (
    PropertyId  INTEGER NOT NULL REFERENCES Properties(PropertyId),
    IntValue    INTEGER NOT NULL,

    PRIMARY KEY (PropertyId)
);

/**
 * Properties that apply to the whole document with float values
 **/
CREATE TABLE DocumentFloatProperties (
    PropertyId  INTEGER NOT NULL REFERENCES Properties(PropertyId),
    FloatValue  FLOAT   NOT NULL,

    PRIMARY KEY (PropertyId)
);

/**
 * Properties that apply to the whole document but are encoded as postcard blob values (serialized `CanvasProperty` values)
 **/
CREATE TABLE DocumentBlobProperties (
    PropertyId  INTEGER NOT NULL REFERENCES Properties(PropertyId),
    BlobValue   BLOB    NOT NULL,

    PRIMARY KEY (PropertyId)
);

/**
 * The layers that make up this document
 **/
CREATE TABLE Layers (
    LayerId     INTEGER     NOT NULL PRIMARY KEY AUTOINCREMENT,
    LayerGuid   CHAR(36)    NOT NULL,
    OrderIdx    INTEGER     NOT NULL
);

/**
 * Integer properties attached to a layer
 **/
CREATE TABLE LayerIntProperties (
    LayerId     INTEGER NOT NULL REFERENCES Layers(LayerId) ON DELETE CASCADE,
    PropertyId  INTEGER NOT NULL REFERENCES Properties(PropertyId),
    IntValue    INTEGER NOT NULL,

    PRIMARY KEY (LayerId, PropertyId)
);

/**
 * Float properties attached to a layer
 **/
CREATE TABLE LayerFloatProperties (
    LayerId     INTEGER NOT NULL REFERENCES Layers(LayerId) ON DELETE CASCADE,
    PropertyId  INTEGER NOT NULL REFERENCES Properties(PropertyId),
    FloatValue  FLOAT   NOT NULL,

    PRIMARY KEY (LayerId, PropertyId)
);

/**
 * Blob properties attached to a layer (postcard serialized `CanvasProperty` values)
 **/
CREATE TABLE LayerBlobProperties (
    LayerId     INTEGER NOT NULL REFERENCES Layers(LayerId) ON DELETE CASCADE,
    PropertyId  INTEGER NOT NULL REFERENCES Properties(PropertyId),
    BlobValue   BLOB    NOT NULL,

    PRIMARY KEY (LayerId, PropertyId)
);

/**
 * The shapes that are in the canvas, as a list
 *
 * Shape type can be:
 *  0 - bezier path
 *  1 - rectangle
 *  2 - ellipse
 *  3 - polygon
 *  4 - group
 **/
CREATE TABLE Shapes (
    ShapeId         INTEGER     NOT NULL,
    ShapeGuid       CHAR(36)    NOT NULL,
    ShapeType       INTEGER     NOT NULL REFERENCES ShapeTypes(ShapeTypeId),
    ShapeDataType   INTEGER     NOT NULL,
    ShapeData       BLOB        NOT NULL,

    PRIMARY KEY (ShapeId)
) WITHOUT ROWID;

/**
 * For shapes that are on a layer, this associates them with that layer
 * 
 * If a shape is part of a group, it's also stored in order on the layer (so for things like rendering, it's not necessary to know about groups)
 **/
CREATE TABLE ShapeLayers (
    ShapeId     INTEGER NOT NULL REFERENCES Shapes(ShapeId) ON DELETE CASCADE,
    LayerId     INTEGER NOT NULL REFERENCES Layers(LayerId) ON DELETE CASCADE,
    OrderIdx    INTEGER NOT NULL,
    Time        INTEGER NOT NULL,

    PRIMARY KEY (LayerId, OrderIdx, ShapeId)
) WITHOUT ROWID;

/**
 * For shapes that are part of a group, this associates them with their parent shape
 **/
CREATE TABLE ShapeGroups (
    ShapeId         INTEGER NOT NULL REFERENCES Shapes(ShapeId) ON DELETE CASCADE,
    ParentShapeId   INTEGER NOT NULL REFERENCES Shapes(ShapeId) ON DELETE CASCADE,
    OrderIdx        INTEGER NOT NULL,

    PRIMARY KEY (ShapeId, OrderIdx, ParentShapeId)
);

/**
 * Integer properties attached to a shape
 **/
CREATE TABLE ShapeIntProperties (
    ShapeId     INTEGER NOT NULL REFERENCES Shapes(ShapeId) ON DELETE CASCADE,
    PropertyId  INTEGER NOT NULL REFERENCES Properties(PropertyId),
    IntValue    INTEGER NOT NULL,

    PRIMARY KEY (ShapeId, PropertyId)
);

/**
 * Float properties attached to a shape
 **/
CREATE TABLE ShapeFloatProperties (
    ShapeId     INTEGER NOT NULL REFERENCES Shapes(ShapeId) ON DELETE CASCADE,
    PropertyId  INTEGER NOT NULL REFERENCES Properties(PropertyId),
    FloatValue  FLOAT   NOT NULL,

    PRIMARY KEY (ShapeId, PropertyId)
);

/**
 * Blob properties attached to a shape (postcard serialized `CanvasProperty` values)
 **/
CREATE TABLE ShapeBlobProperties (
    ShapeId     INTEGER NOT NULL REFERENCES Shapes(ShapeId) ON DELETE CASCADE,
    PropertyId  INTEGER NOT NULL REFERENCES Properties(PropertyId),
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
 * Associates brushes with shapes, allowing a shape to take on the properties of a brush
 **/
CREATE TABLE ShapeBrushes (
    ShapeId     INTEGER NOT NULL REFERENCES Shapes(ShapeId) ON DELETE CASCADE,
    BrushId     INTEGER NOT NULL REFERENCES Brushes(BrushId) ON DELETE CASCADE,
    OrderIdx    INTEGER NOT NULL,

    PRIMARY KEY (ShapeId, OrderIdx)
);

/**
 * Integer properties attached to a brush
 **/
CREATE TABLE BrushIntProperties (
    BrushId     INTEGER NOT NULL REFERENCES Brushes(BrushId) ON DELETE CASCADE,
    PropertyId  INTEGER NOT NULL REFERENCES Properties(PropertyId),
    IntValue    INTEGER NOT NULL,

    PRIMARY KEY (BrushId, PropertyId)
);

/**
 * Float properties attached to a brush
 **/
CREATE TABLE BrushFloatProperties (
    BrushId     INTEGER NOT NULL REFERENCES Brushes(BrushId) ON DELETE CASCADE,
    PropertyId  INTEGER NOT NULL REFERENCES Properties(PropertyId),
    FloatValue  FLOAT   NOT NULL,

    PRIMARY KEY (BrushId, PropertyId)
);

/**
 * Blob properties attached to a brush (postcard serialized `CanvasProperty` values)
 **/
CREATE TABLE BrushBlobProperties (
    BrushId     INTEGER NOT NULL REFERENCES Brushes(BrushId) ON DELETE CASCADE,
    PropertyId  INTEGER NOT NULL REFERENCES Properties(PropertyId),
    BlobValue   BLOB    NOT NULL,

    PRIMARY KEY (BrushId, PropertyId)
);


/**
 * Indicates the time that a layer is cleared to create a new frame.
 * 
 * Time is measured in nanoseconds from the start of an animation
 **/
CREATE TABLE LayerFrames (
    LayerId INTEGER NOT NULL REFERENCES Layers(LayerId),
    Time    INTEGER NOT NULL,

    PRIMARY KEY (LayerId, Time)
);

/** 
 * Fast lookup of layers by GUID 
 **/
-- ~50% speedup in benchmark when added (+100% throughput)
CREATE INDEX idx_layers_guid ON Layers(LayerGuid);

/**
 * Fast lookup for how brushes are attached to shapes
 **/
-- Perf improvement not measured by benchmark (doesn't use brushes)
CREATE INDEX idx_shapebrushes_brush ON ShapeBrushes(BrushId);

/**
 * Covering indexes for properties when used on shapes
 **/
-- ~4% speedup when added
CREATE INDEX idx_shape_int_props ON ShapeIntProperties(ShapeId, PropertyId, IntValue);
CREATE INDEX idx_shape_float_props ON ShapeFloatProperties(ShapeId, PropertyId, FloatValue);
CREATE INDEX idx_shape_blob_props ON ShapeBlobProperties(ShapeId, PropertyId, BlobValue);

CREATE INDEX idx_brush_int_props ON BrushIntProperties(BrushId, PropertyId, IntValue);
CREATE INDEX idx_brush_float_props ON BrushFloatProperties(BrushId, PropertyId, FloatValue);
CREATE INDEX idx_brush_blob_props ON BrushBlobProperties(BrushId, PropertyId, BlobValue);

/**
 * Fast lookup of shapes by GUID
 **/
CREATE INDEX idx_shapes_guid ON Shapes(ShapeGuid);

/**
 * Fast lookup of shape layer assignments by ShapeId
 **/
CREATE INDEX idx_shapelayers_shape ON ShapeLayers(ShapeId);

CREATE INDEX idx_shape_time ON ShapeLayers(LayerId, Time, OrderIdx, ShapeId);
CREATE INDEX idx_layer_time ON LayerFrames(LayerId, Time);

