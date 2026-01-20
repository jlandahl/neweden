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
                SELECT solarSystemID, solarSystemName, s.x, s.y, s.z, security, regionName
                FROM mapSolarSystems s
                JOIN mapRegions r USING (regionID)
                ",
            )?;

            stm.query([])?
                .mapped(|row| {
                    Ok(types::System {
                        id: row.get::<_, u32>(0)?.into(),
                        name: row.get(1)?,
                        coordinate: (row.get(2)?, row.get(3)?, row.get(4)?).into(),
                        security: row.get::<_, f32>(5)?.into(),
                        region_name: row.get(6)?,
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
                    toRegionID,
                    toConstellationID,
                    toSolarSystemID
                FROM mapSolarSystemJumps
                ",
            )?;

            stm.query([])?
                .mapped(|row| {
                    let from_region: i32 = row.get(0)?;
                    let from_constellation: i32 = row.get(1)?;
                    let from_system: i32 = row.get(2)?;

                    let to_region: i32 = row.get(3)?;
                    let to_constellation: i32 = row.get(4)?;
                    let to_system: i32 = row.get(5)?;

                    let stargate_type = if from_region != to_region {
                        types::StargateType::Regional
                    } else if from_constellation != to_constellation {
                        types::StargateType::Constellation
                    } else {
                        types::StargateType::Local
                    };

                    Ok(types::Connection {
                        from: from_system.into(),
                        to: to_system.into(),
                        r#type: types::ConnectionType::Stargate(stargate_type),
                    })
                })
                .collect::<Result<Vec<_>, _>>()?
        };

        Ok(types::Universe::new(systems.into(), connections.into()))
    }
}
