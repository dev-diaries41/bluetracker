/// Migrates the database to version 2 by updating each device's `manufacturer_id`
/// based on the most recent detection's `manufacturer_data`.
// pub fn migrate_to_v2(conn: &mut Connection) -> Result<()> {
//     let tx = conn.transaction()?;

//     // Retrieve all device addresses in a separate block so the statement is dropped afterward.
//     let addresses: Vec<String> = {
//         let mut stmt = tx.prepare("SELECT address FROM devices")?;
//         let addresses_iter = stmt.query_map([], |row| row.get(0))?;
//         addresses_iter.collect::<Result<Vec<_>>>()?
//     };

//     // Iterate over each device address.
//     for address in addresses {
//         // Retrieve the most recent detection's manufacturer_data for this device.
//         let manufacturer_data: Option<String> = {
//             let mut det_stmt = tx.prepare(
//                 "SELECT manufacturer_data 
//                  FROM detections 
//                  WHERE device_address = ? 
//                  ORDER BY timestamp DESC 
//                  LIMIT 1",
//             )?;
//             det_stmt
//                 .query_row(params![address], |row| row.get(0))
//                 .optional()?
//         };

//         // If a detection was found, compute the manufacturer_id and update the device.
//         if let Some(data) = manufacturer_data {
//             let manufacturer_id = get_manufacturer_id(&data);
//             tx.execute(
//                 "UPDATE devices SET manufacturer_id = ? WHERE address = ?",
//                 params![manufacturer_id, address],
//             )?;
//         }
//     }

//     // All statements are now dropped, so we can safely commit.
//     tx.commit()?;
//     Ok(())
// }
