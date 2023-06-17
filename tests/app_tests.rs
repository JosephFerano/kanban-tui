#[cfg(test)]
mod app_tests {
    use anyhow::{Error};
    use kanban_tui::State;
    use rusqlite::Connection;

    fn create_connection() -> Result<Connection, Error> {
        let mut conn = Connection::open_in_memory()?;

        let migrations = include_str!("../sql/migrations.sql");
        let migrations: Vec<&str> = migrations.split(';').collect();
        let tx = conn.transaction()?;
        for m in migrations {
            if !m.trim().is_empty() {
                tx.execute(m, ())?;
            }
        }
        tx.commit()?;
        Ok(conn)
    }

    #[test]
    fn it_adds_tasks_to_different_columns() -> Result<(), Error> {
        let mut state = State::new(create_connection()?)?;

        state.add_new_task(String::from("T1"), String::from("D1"))?;
        state.add_new_task(String::from("T2"), String::from("D2"))?;
        state.select_column_right()?;
        state.select_column_right()?;
        state.add_new_task(String::from("T3"), String::from("D3"))?;
        state.select_column_left()?;
        state.add_new_task(String::from("T4"), String::from("D4"))?;

        assert_eq!(state.columns[0].tasks.len(), 2);
        assert_eq!(state.columns[1].tasks.len(), 1);
        assert_eq!(state.columns[2].tasks.len(), 1);
        assert_eq!(&state.columns[0].tasks[0].title, "T1");
        assert_eq!(&state.columns[0].tasks[1].description, "D2");
        assert_eq!(&state.columns[1].tasks[0].title, "T4");
        assert_eq!(&state.columns[2].tasks[0].description, "D3");

        // Reload the data from the database then rerun the asserts to
        // make sure everything was saved correctly
        let state = State::new(state.db_conn.0)?;

        assert_eq!(state.columns[0].tasks.len(), 2);
        assert_eq!(state.columns[1].tasks.len(), 1);
        assert_eq!(state.columns[2].tasks.len(), 1);
        assert_eq!(&state.columns[0].tasks[0].title, "T1");
        assert_eq!(&state.columns[0].tasks[1].description, "D2");
        assert_eq!(&state.columns[1].tasks[0].title, "T4");
        assert_eq!(&state.columns[2].tasks[0].description, "D3");

        Ok(())
    }

    #[test]
    fn it_selects_the_correct_tasks_and_stays_inbound() -> Result<(), Error> {
        let mut state = State::new(create_connection()?)?;

        state.move_task_up()?;
        state.move_task_up()?;
        state.move_task_down()?;
        state.move_task_down()?;
        for _ in 0..10 {
            state.select_column_right()?;
        }
        for _ in 0..10 {
            state.select_column_left()?;
        }
        state.add_new_task(String::from("T1"), String::from("D1"))?;
        assert_eq!(state.get_selected_task().unwrap().title, "T1");
        state.add_new_task(String::from("T2"), String::from("D2"))?;
        assert_eq!(state.get_selected_task().unwrap().title, "T2");
        state.select_task_above()?;
        assert_eq!(state.get_selected_task().unwrap().title, "T1");
        for _ in 0..6 {
            state.select_column_right()?;
        }
        state.select_column_left()?;
        state.add_new_task(String::from("T3"), String::from("D3"))?;
        assert_eq!(state.get_selected_task().unwrap().title, "T3");
        assert_eq!(state.get_selected_column().name, "Done");
        for _ in 0..6 {
            state.select_column_right()?;
        }
        for _ in 0..4 {
            state.select_column_left()?;
        }
        assert_eq!(state.get_selected_task().unwrap().title, "T1");
        state.select_column_right()?;
        state.select_column_right()?;


        // Reload the data from the database then rerun the asserts to
        // make sure everything was saved correctly
        let mut state = State::new(state.db_conn.0)?;

        assert_eq!(state.get_selected_task().unwrap().title, "T3");
        state.select_task_below()?;
        state.select_task_below()?;
        assert_eq!(state.get_selected_task().unwrap().title, "T3");
        state.select_column_left()?;
        state.select_column_left()?;
        assert_eq!(state.get_selected_task().unwrap().title, "T1");
        state.select_task_below()?;
        assert_eq!(state.get_selected_task().unwrap().title, "T2");

        Ok(())
    }

    #[test]
    fn it_selects_the_correct_first_and_last_task() -> Result<(), Error> {
        let mut state = State::new(create_connection()?)?;

        for i in 1..11 {
            state.add_new_task(format!("T{i}"), format!("D{i}"))?;
        }
        assert_eq!(state.get_selected_task().unwrap().title, "T10");
        state.select_last_task()?;
        assert_eq!(state.get_selected_task().unwrap().title, "T10");
        state.select_first_task()?;
        assert_eq!(state.get_selected_task().unwrap().title, "T1");
        for _ in 0..10 {
            state.move_task_down()?;
        }
        assert_eq!(state.get_selected_task().unwrap().title, "T1");
        state.select_first_task()?;
        assert_eq!(state.get_selected_task().unwrap().title, "T2");
        state.select_last_task()?;
        state.select_task_above()?;
        state.select_task_above()?;
        assert_eq!(state.get_selected_task().unwrap().title, "T9");
        for _ in 0..10 {
            state.move_task_up()?;
        }
        state.select_first_task()?;
        assert_eq!(state.get_selected_task().unwrap().title, "T9");
        state.select_last_task()?;
        assert_eq!(state.get_selected_task().unwrap().title, "T1");

        // Reload the data from the database then rerun the asserts to
        // make sure everything was saved correctly
        let mut state = State::new(state.db_conn.0)?;

        assert_eq!(state.get_selected_task().unwrap().title, "T1");
        state.select_last_task()?;
        assert_eq!(state.get_selected_task().unwrap().title, "T1");
        state.select_first_task()?;
        assert_eq!(state.get_selected_task().unwrap().title, "T9");


        Ok(())
    }

    #[test]
    fn it_moves_tasks_up_and_down_and_to_different_columns() -> Result<(), Error> {
        let mut state = State::new(create_connection()?)?;

        state.add_new_task(String::from("T1"), String::from("D1"))?;
        state.add_new_task(String::from("T2"), String::from("D2"))?;

        state.select_task_above()?;
        state.move_task_up()?;
        state.move_task_down()?;
        state.move_task_down()?;

        assert_eq!(&state.columns[0].tasks[1].title, "T1");
        assert_eq!(&state.columns[0].tasks[0].title, "T2");
        state.select_column_right()?;
        state.add_new_task(String::from("T3"), String::from("D3"))?;
        state.move_task_column_right()?;
        assert_eq!(state.columns[1].tasks.len(), 0);
        assert_eq!(state.columns[2].tasks.len(), 1);

        for _ in 0..5 {
            state.move_task_column_right()?;
        }
        for _ in 0..4 {
            state.move_task_column_left()?;
        }

        assert_eq!(state.columns[0].tasks.len(), 3);
        assert_eq!(state.columns[1].tasks.len(), 0);
        assert_eq!(state.columns[2].tasks.len(), 0);
        assert_eq!(state.columns[3].tasks.len(), 0);
        assert_eq!(state.get_selected_task().unwrap().title, "T3");
        state.select_task_below()?;
        state.select_task_above()?;
        assert_eq!(state.get_selected_task().unwrap().title, "T1");
        state.select_task_above()?;
        assert_eq!(state.get_selected_task().unwrap().title, "T2");
        state.select_first_task()?;
        assert_eq!(state.get_selected_task().unwrap().title, "T2");
        state.select_last_task()?;
        assert_eq!(state.get_selected_task().unwrap().title, "T3");
        state.select_task_above()?;

        // Reload the data from the database then rerun the asserts to
        // make sure everything was saved correctly
        let mut state = State::new(state.db_conn.0)?;

        assert_eq!(state.get_selected_task().unwrap().title, "T1");
        state.select_task_below()?;
        assert_eq!(state.get_selected_task().unwrap().title, "T3");
        state.select_task_below()?;
        assert_eq!(state.get_selected_task().unwrap().title, "T3");
        state.select_first_task()?;
        assert_eq!(state.get_selected_task().unwrap().title, "T2");
        state.select_last_task()?;
        assert_eq!(state.get_selected_task().unwrap().title, "T3");
        assert_eq!(state.columns[0].tasks.len(), 3);
        assert_eq!(state.columns[1].tasks.len(), 0);
        assert_eq!(state.columns[2].tasks.len(), 0);
        assert_eq!(state.columns[3].tasks.len(), 0);

        Ok(())
    }

    #[test]
    fn it_edits_a_task_and_updates_it() -> Result<(), Error> {
        let mut state = State::new(create_connection()?)?;

        state.add_new_task(String::from("T1"), String::from("D1"))?;
        assert_eq!(state.get_selected_task().unwrap().title, "T1");
        assert_eq!(state.get_selected_task().unwrap().description, "D1");
        state.edit_task(String::from("T1"), String::from("D2"))?;
        assert_eq!(state.get_selected_task().unwrap().title, "T1");
        assert_eq!(state.get_selected_task().unwrap().description, "D2");
        state.edit_task(String::from("T2"), String::from("D1"))?;
        assert_eq!(state.get_selected_task().unwrap().title, "T2");
        assert_eq!(state.get_selected_task().unwrap().description, "D1");
        for _ in 0..4 {
            state.move_task_column_right()?;
        }
        assert_eq!(state.get_selected_task().unwrap().title, "T2");
        assert_eq!(state.get_selected_task().unwrap().description, "D1");
        state.edit_task(String::from("T3"), String::from("D3"))?;
        assert_eq!(state.get_selected_task().unwrap().title, "T3");
        assert_eq!(state.get_selected_task().unwrap().description, "D3");
        for _ in 0..4 {
            state.move_task_column_left()?;
        }
        assert_eq!(state.get_selected_task().unwrap().title, "T3");
        assert_eq!(state.get_selected_task().unwrap().description, "D3");
        state.edit_task(String::from("T3"), String::from("D3"))?;

        assert_eq!(state.columns[0].tasks.len(), 1);
        assert_eq!(state.columns[1].tasks.len(), 0);
        assert_eq!(state.columns[2].tasks.len(), 0);
        assert_eq!(state.columns[3].tasks.len(), 0);

        // Reload the data from the database then rerun the asserts to
        // make sure everything was saved correctly
        let state = State::new(state.db_conn.0)?;

        assert_eq!(state.get_selected_task().unwrap().title, "T3");
        assert_eq!(state.get_selected_task().unwrap().description, "D3");

        Ok(())
    }

    #[test]
    fn it_deletes_a_task() -> Result<(), Error> {
        let mut state = State::new(create_connection()?)?;

        state.add_new_task(String::from("T1"), String::from("D1"))?;
        assert_eq!(state.get_selected_task().unwrap().title, "T1");
        for col in state.columns.iter().skip(1) {
            assert_eq!(col.tasks.len(), 0);
        }
        state.delete_task()?;
        assert!(state.get_selected_task().is_none());
        for col in &state.columns {
            assert_eq!(col.tasks.len(), 0);
        }
        state.delete_task()?;

        state.add_new_task(String::from("T1"), String::from("D1"))?;
        state.add_new_task(String::from("T2"), String::from("D2"))?;
        state.add_new_task(String::from("T3"), String::from("D3"))?;
        assert_eq!(state.get_selected_task().unwrap().title, "T3");
        state.delete_task()?;
        assert_eq!(state.get_selected_task().unwrap().title, "T2");
        state.add_new_task(String::from("T3"), String::from("D3"))?;
        state.select_task_above()?;
        assert_eq!(state.get_selected_task().unwrap().title, "T2");
        state.delete_task()?;
        state.select_task_above()?;
        assert_eq!(state.get_selected_task().unwrap().title, "T1");
        state.select_last_task()?;
        assert_eq!(state.get_selected_task().unwrap().title, "T3");
        state.add_new_task(String::from("T2"), String::from("D2"))?;
        for _ in 0..4 {
            state.move_task_up()?;
        }
        state.delete_task()?;
        assert_eq!(state.get_selected_task().unwrap().title, "T1");
        state.select_task_below()?;
        assert_eq!(state.get_selected_task().unwrap().title, "T3");
        for _ in 0..4 {
            state.delete_task()?;
        }
        assert!(state.get_selected_task().is_none());
        for col in &state.columns {
            assert_eq!(col.tasks.len(), 0);
        }

        // Reload the data from the database then rerun the asserts to
        // make sure everything was saved correctly
        let state = State::new(state.db_conn.0)?;

        assert!(state.get_selected_task().is_none());
        for col in state.columns {
            assert_eq!(col.tasks.len(), 0);
        }

        Ok(())
    }
}
