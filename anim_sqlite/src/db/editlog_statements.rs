use super::*;
use rusqlite::*;

///
/// Prepared statement cache for the database
/// 
pub struct DbStatements<'a> {
    /// Connection to the database
    sqlite:             &'a Connection,

    // Edit log
    
    insert_editlog:             Option<CachedStatement<'a>>,
    insert_el_size:             Option<CachedStatement<'a>>,
    insert_el_layer:            Option<CachedStatement<'a>>,
    insert_el_when:             Option<CachedStatement<'a>>,
    insert_el_brush_type:       Option<CachedStatement<'a>>,
    insert_el_brush:            Option<CachedStatement<'a>>,
    insert_el_brush_ink:        Option<CachedStatement<'a>>,
    insert_el_brush_properties: Option<CachedStatement<'a>>,
    insert_el_color_type:       Option<CachedStatement<'a>>,
    insert_el_color_rgb:        Option<CachedStatement<'a>>,
    insert_el_color_hsluv:      Option<CachedStatement<'a>>,
    insert_el_rawpoint:         Option<CachedStatement<'a>>,
}

impl<'a> DbStatements<'a> {
    ///
    /// Creates a new DB statements cache
    ///
    pub fn new(connection: &'a Connection) -> DbStatements<'a> {
        DbStatements {
            sqlite:             connection,

            insert_editlog:             None,
            insert_el_size:             None,
            insert_el_layer:            None,
            insert_el_when:             None,
            insert_el_brush_type:       None,
            insert_el_brush:            None,
            insert_el_brush_ink:        None,
            insert_el_brush_properties: None,
            insert_el_color_type:       None,
            insert_el_color_rgb:        None,
            insert_el_color_hsluv:      None,
            insert_el_rawpoint:         None
        }
    }

    pub fn insert_editlog<'b>(&'b mut self) -> &'b mut CachedStatement<'a> {
        let sqlite = &self.sqlite;
        self.insert_editlog.get_or_insert_with(|| 
            sqlite.prepare_cached("INSERT INTO Flo_EditLog (Edit) VALUES (?)").unwrap()
        )
    }

    pub fn insert_el_size<'b>(&'b mut self) -> &'b mut CachedStatement<'a> {
        let sqlite = &self.sqlite;
        self.insert_el_size.get_or_insert_with(|| 
            sqlite.prepare_cached("INSERT INTO Flo_EL_Size (EditId, X, Y) VALUES (?, ?, ?)").unwrap()
        )
    }

    pub fn insert_el_layer<'b>(&'b mut self) -> &'b mut CachedStatement<'a> {
        let sqlite = &self.sqlite;
        self.insert_el_layer.get_or_insert_with(|| 
            sqlite.prepare_cached("INSERT INTO Flo_EL_Layer (EditId, Layer) VALUES (?, ?)").unwrap()
        )
    }

    pub fn insert_el_when<'b>(&'b mut self) -> &'b mut CachedStatement<'a> {
        let sqlite = &self.sqlite;
        self.insert_el_when.get_or_insert_with(|| 
            sqlite.prepare_cached("INSERT INTO Flo_EL_When (EditId, AtTime) VALUES (?, ?)").unwrap()
        )
    }

    pub fn insert_el_brush_type<'b>(&'b mut self) -> &'b mut CachedStatement<'a> {
        let sqlite = &self.sqlite;
        self.insert_el_brush_type.get_or_insert_with(|| 
            sqlite.prepare_cached("INSERT INTO Flo_EL_Brush_Type (BrushType) VALUES (?)").unwrap()
        )
    }

    pub fn insert_el_brush<'b>(&'b mut self) -> &'b mut CachedStatement<'a> {
        let sqlite = &self.sqlite;
        self.insert_el_brush.get_or_insert_with(|| 
            sqlite.prepare_cached("INSERT INTO Flo_EL_Brush (EditId, DrawingStyle, BrushType, Brush) VALUES (?, ?, ?, ?)").unwrap()
        )
    }

    pub fn insert_el_brush_ink<'b>(&'b mut self) -> &'b mut CachedStatement<'a> {
        let sqlite = &self.sqlite;
        self.insert_el_brush_ink.get_or_insert_with(|| 
            sqlite.prepare_cached("INSERT INTO Flo_EL_Brush_Ink (Brush, MinWidth, MaxWidth, ScaleUpDistance) VALUES (?, ?, ?, ?)").unwrap()
        )
    }

    pub fn insert_el_brush_properties<'b>(&'b mut self) -> &'b mut CachedStatement<'a> {
        let sqlite = &self.sqlite;
        self.insert_el_brush_properties.get_or_insert_with(|| 
            sqlite.prepare_cached("INSERT INTO Flo_EL_BrushProperties (EditId, Size, Opacity, Color) VALUES (?, ?, ?, ?)").unwrap()
        )
    }

    pub fn insert_el_color_type<'b>(&'b mut self) -> &'b mut CachedStatement<'a> {
        let sqlite = &self.sqlite;
        self.insert_el_brush_type.get_or_insert_with(|| 
            sqlite.prepare_cached("INSERT INTO Flo_EL_Color_Type (ColorType) VALUES (?)").unwrap()
        )
    }

    pub fn insert_el_color_rgb<'b>(&'b mut self) -> &'b mut CachedStatement<'a> {
        let sqlite = &self.sqlite;
        self.insert_el_color_rgb.get_or_insert_with(|| 
            sqlite.prepare_cached("INSERT INTO Flo_EL_Color_Rgb (Color, R, G, B) VALUES (?, ?, ?, ?)").unwrap()
        )
    }

    pub fn insert_el_color_hsluv<'b>(&'b mut self) -> &'b mut CachedStatement<'a> {
        let sqlite = &self.sqlite;
        self.insert_el_color_hsluv.get_or_insert_with(|| 
            sqlite.prepare_cached("INSERT INTO Flo_EL_Color_Hsluv (Color, H, S, L) VALUES (?, ?, ?, ?)").unwrap()
        )
    }

    pub fn insert_el_rawpoint<'b>(&'b mut self) -> &'b mut CachedStatement<'a> {
        let sqlite = &self.sqlite;
        self.insert_el_rawpoint.get_or_insert_with(|| 
            sqlite.prepare_cached("INSERT INTO Flo_EL_RawPoint (EditId, PointId, PosX, PosY, Pressure, TiltX, TiltY) VALUES (?, ?, ?, ?, ?, ?, ?)").unwrap()
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// Creates a DB core to test the SQL statements on
    fn core() -> AnimationDbCore {
        let mut core = AnimationDbCore::new(Connection::open_in_memory().unwrap());
        core.setup();

        core
    }

    #[test]
    fn insert_editlog() {
        let core     = core();
        let mut stmt = DbStatements::new(&core.sqlite);

        let row_id = stmt.insert_editlog().insert(&[&2]).unwrap();
        assert!(row_id == 1);
    }

    #[test]
    fn insert_el_size() {
        let core     = core();
        let mut stmt = DbStatements::new(&core.sqlite);

        let edit_id = stmt.insert_editlog().insert(&[&0]).unwrap();
        stmt.insert_el_size().insert(&[&edit_id, &1980.0, &1080.0]).unwrap();
    }

    #[test]
    fn insert_el_layer() {
        let core     = core();
        let mut stmt = DbStatements::new(&core.sqlite);

        let edit_id = stmt.insert_editlog().insert(&[&0]).unwrap();
        stmt.insert_el_layer().insert(&[&edit_id, &1]).unwrap();
    }

    #[test]
    fn insert_el_when() {
        let core     = core();
        let mut stmt = DbStatements::new(&core.sqlite);

        let edit_id = stmt.insert_editlog().insert(&[&0]).unwrap();
        stmt.insert_el_when().insert(&[&edit_id, &1000000]).unwrap();
    }

    #[test]
    fn insert_el_brush_type() {
        let core     = core();
        let mut stmt = DbStatements::new(&core.sqlite);

        let brush_id = stmt.insert_el_brush_type().insert(&[&0]).unwrap();
        assert!(brush_id == 1);
    }

    #[test]
    fn insert_el_color_type() {
        let core     = core();
        let mut stmt = DbStatements::new(&core.sqlite);

        let color_id = stmt.insert_el_color_type().insert(&[&0]).unwrap();
        assert!(color_id == 1);
    }

    #[test]
    fn insert_el_brush_ink() {
        let core     = core();
        let mut stmt = DbStatements::new(&core.sqlite);

        let brush_id = stmt.insert_el_brush_type().insert(&[&0]).unwrap();
        stmt.insert_el_brush_ink().insert(&[&brush_id, &10.0, &20.0, &30.0]).unwrap();
    }

    #[test]
    fn insert_el_brush_properties() {
        let core     = core();
        let mut stmt = DbStatements::new(&core.sqlite);

        let edit_id = stmt.insert_editlog().insert(&[&0]).unwrap();
        let color_id = stmt.insert_el_color_type().insert(&[&0]).unwrap();
        stmt.insert_el_brush_properties().insert(&[&edit_id, &20.0, &1.0, &color_id]).unwrap();
    }

    #[test]
    fn insert_el_color_rgb() {
        let core     = core();
        let mut stmt = DbStatements::new(&core.sqlite);

        let color_id = stmt.insert_el_color_type().insert(&[&0]).unwrap();
        stmt.insert_el_color_rgb().insert(&[&color_id, &1.0, &0.0, &1.0]).unwrap();
    }

    #[test]
    fn insert_el_color_hsluv() {
        let core     = core();
        let mut stmt = DbStatements::new(&core.sqlite);

        let color_id = stmt.insert_el_color_type().insert(&[&0]).unwrap();
        stmt.insert_el_color_hsluv().insert(&[&color_id, &45.0, &100.0, &50.0]).unwrap();
    }

    #[test]
    fn insert_el_rawpoint() {
        let core     = core();
        let mut stmt = DbStatements::new(&core.sqlite);

        let edit_id = stmt.insert_editlog().insert(&[&0]).unwrap();
        stmt.insert_el_rawpoint().insert(&[&edit_id, &0, &1.0, &2.0, &0.5, &0.0, &0.0]).unwrap();
        stmt.insert_el_rawpoint().insert(&[&edit_id, &1, &1.0, &2.0, &0.5, &0.0, &0.0]).unwrap();
        stmt.insert_el_rawpoint().insert(&[&edit_id, &2, &1.0, &2.0, &0.5, &0.0, &0.0]).unwrap();
    }
}
