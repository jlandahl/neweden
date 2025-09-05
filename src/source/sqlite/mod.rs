use anyhow;
use rusqlite;

use crate::types;

pub struct DatabaseBuilder {
    uri: String,
}

/// Loads a universe from a database.
///
/// `Universe` implements `Navigatable` and can be used in pathfinding.
///
/// `Universe` is intended to be used immutable and can only be instantiated
/// from a data source such as a database. If you need to add additional connections,
/// such as dynamic wormhole connections during pathfinding, construct an `ExtendedUniverse`
/// from a universe by calling `.extend()` or `ExtendedUniverse::new()`.
///
/// # Example
/// ```
/// use std::env;
/// use neweden::source::sqlite::DatabaseBuilder;
/// use neweden::Navigatable;
///
/// let uri = std::env::var("SQLITE_URI").unwrap();
/// let universe = DatabaseBuilder::new(&uri).build().unwrap();
/// let system_id = 30000142.into(); // returns a SystemId
/// println!("{:?}", universe.get_system(&system_id).unwrap().name); // Jita
/// ```
impl DatabaseBuilder {
    pub fn new(uri: &str) -> Self {
        Self {
            uri: uri.to_string(),
        }
    }

    pub fn build(self) -> anyhow::Result<types::Universe> {
        Self::from_connection(rusqlite::Connection::open_with_flags(
            self.uri,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
        )?)
    }

    pub(self) fn from_connection(conn: rusqlite::Connection) -> anyhow::Result<types::Universe> {
        let systems = {
            let mut stm = conn.prepare(
                "
                SELECT solarSystemID, solarSystemName, x, y, z, security
                FROM mapSolarSystems
                ",
            )?;

            stm.query([])?
                .mapped(|row| {
                    Ok(types::System {
                        id: row.get::<_, u32>(0)?.into(),
                        name: row.get(1)?,
                        coordinate: (row.get(2)?, row.get(3)?, row.get(4)?).into(),
                        security: row.get::<_, f32>(5)?.into(),
                    })
                })
                .collect::<Result<Vec<_>, _>>()?
        };

        let connections = {
            let mut stm = conn.prepare(
                "
                SELECT
                    fromRegionID,
                    fromConstellationID,
                    fromSolarSystemID,
                    toSolarSystemID
                    toConstellationID,
                    toRegionID
                FROM mapSolarSystemJumps
                ",
            )?;

            stm.query([])?
                .mapped(|row| {
                    let from: i32 = row.get(2)?;
                    let to: i32 = row.get(3)?;
                    let stargate_type = match (
                        row.get::<_, i32>(0),
                        row.get::<_, i32>(1),
                        row.get::<_, i32>(4),
                        row.get::<_, i32>(5),
                    ) {
                        (a, _, _, b) if a != b => types::StargateType::Regional,
                        (_, a, b, _) if a != b => types::StargateType::Constellation,
                        _ => types::StargateType::Local,
                    };
                    Ok(types::Connection {
                        from: from.into(),
                        to: to.into(),
                        r#type: types::ConnectionType::Stargate(stargate_type),
                    })
                })
                .collect::<Result<Vec<_>, _>>()?
        };

        Ok(types::Universe::new(systems.into(), connections.into()))
    }
}
