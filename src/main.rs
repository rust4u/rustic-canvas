use chrono::{DateTime, Utc};
use rand::{seq::SliceRandom, thread_rng, Rng};
use std::{
    sync::{Arc, Mutex},
    thread,
};

const TOTAL_ARTISTS: usize = 1;
const TOTAL_ITEMS: usize = 10;
const TOTAL_WEIGHT_KG: usize = 10;
const MIN_REQUIRED_TOOLS: usize = 2;
const MAX_ALLOWED_TOOLS: usize = 5;

#[derive(Debug)]
struct SharedResources {
    tools: Vec<(String, usize)>,
    paints: Vec<(String, usize)>,
}

impl SharedResources {
    fn default() -> Self {
        Self {
            tools: vec![
                ("brush".to_string(), TOTAL_ITEMS),
                ("palette".to_string(), TOTAL_ITEMS),
                ("canvas".to_string(), TOTAL_ITEMS),
                ("eraser".to_string(), TOTAL_ITEMS),
                ("sponges".to_string(), TOTAL_ITEMS),
                ("roller".to_string(), TOTAL_ITEMS),
                ("sculpting tool".to_string(), TOTAL_ITEMS),
                ("water container".to_string(), TOTAL_ITEMS),
                ("rags".to_string(), TOTAL_ITEMS),
                ("tape".to_string(), TOTAL_ITEMS),
            ],
            paints: vec![
                ("red".to_string(), TOTAL_WEIGHT_KG),
                ("blue".to_string(), TOTAL_WEIGHT_KG),
                ("green".to_string(), TOTAL_WEIGHT_KG),
                ("yellow".to_string(), TOTAL_WEIGHT_KG),
                ("black".to_string(), TOTAL_WEIGHT_KG),
                ("white".to_string(), TOTAL_WEIGHT_KG),
                ("purple".to_string(), TOTAL_WEIGHT_KG),
                ("orange".to_string(), TOTAL_WEIGHT_KG),
                ("pink".to_string(), TOTAL_WEIGHT_KG),
                ("brown".to_string(), TOTAL_WEIGHT_KG),
            ],
        }
    }

    fn take_out_resources(&mut self, tools: Vec<String>) {
        for tool in tools {
            if let Some(pos) = self.tools.iter().position(|(name, _)| *name == tool) {
                let (_, quantity) = &mut self.tools[pos];
                *quantity -= 1;
                if *quantity == 0 {
                    self.tools.remove(pos);
                }
            } else {
                println!("Warning: Tool '{}' not found in resources.", tool);
            }
        }
    }
}

enum State {
    TakeOut,
    Return,
    Fill,
    Change,
    New,
    Retire,
    Damage,
    Lost,
    Audit,
    Reserved,
    Repair,
    Expired,
    Sold,
}

struct ArtistToolPreferences {
    artist_id: usize,
    preferred_tools: Vec<String>,
    datetime: Option<DateTime<Utc>>,
    state: Option<State>,
}

impl ArtistToolPreferences {
    fn default() -> Self {
        Self {
            artist_id: 0,
            preferred_tools: vec![],
            datetime: None,
            state: None,
        }
    }
}

struct ArtistToolRegistry {
    artist_tool_preferences: Vec<ArtistToolPreferences>,
    shared_resources: Arc<Mutex<SharedResources>>,
}

impl ArtistToolRegistry {
    fn new(resources: &Arc<Mutex<SharedResources>>) -> Self {
        Self {
            artist_tool_preferences: vec![],
            shared_resources: Arc::clone(resources),
        }
    }

    fn tool_registry(&mut self, id: usize, tools: Vec<String>) {
        self.artist_tool_preferences.push(ArtistToolPreferences {
            artist_id: id,
            datetime: Some(Utc::now()),
            state: Some(State::TakeOut),
            preferred_tools: tools.clone(),
        });

        // Lock shared resources and update them
        if let Ok(mut update_resources) = self.shared_resources.lock() {
            update_resources.take_out_resources(tools);
        } else {
            println!("Error: Unable to lock shared resources.");
        }
    }
}

fn artis_task(
    artist_tool_registry: Arc<Mutex<ArtistToolRegistry>>,
    id: usize,
    resources: Arc<Mutex<SharedResources>>,
) {
    let artist_tools: (usize, Vec<String>);
    {
        let resources = resources.lock().expect("Failed to lock resources");
        artist_tools = tools_usage(id, &resources.tools);
    }

    let mut registry = artist_tool_registry
        .lock()
        .expect("Failed to lock registry");
    registry.tool_registry(artist_tools.0, artist_tools.1);

    #[cfg(debug_assertions)]
    simulate_task_delay();
}

fn tools_usage(id: usize, tools: &Vec<(String, usize)>) -> (usize, Vec<String>) {
    let mut rng = thread_rng();
    let tool_count = rng.gen_range(MIN_REQUIRED_TOOLS..=MAX_ALLOWED_TOOLS);
    let selected_tools: Vec<_> = tools.choose_multiple(&mut rng, tool_count).collect();

    let string_values: Vec<String> = selected_tools.iter().map(|&(ref s, _)| s.clone()).collect();
    println!("Artist {}: Selected tools: {:#?}", id, string_values);
    (id, string_values)
}

fn simulate_task_delay() {
    thread::sleep(std::time::Duration::from_millis(10));
}

fn main() {
    let resources = SharedResources::default();
    let shared_resources = Arc::new(Mutex::new(resources));
    let artist_tool_registry = Arc::new(Mutex::new(ArtistToolRegistry::new(&shared_resources)));

    let mut handles = vec![];

    for id in 0..TOTAL_ARTISTS {
        let resources_arc_clone = Arc::clone(&shared_resources);
        let artist_tool_registry_arc_clone = Arc::clone(&artist_tool_registry);
        let handle = thread::spawn(move || {
            artis_task(artist_tool_registry_arc_clone, id, resources_arc_clone)
        });
        handles.push(handle)
    }

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    println!("End");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shared_resources_initialization() {
        let resources = SharedResources::default();
        assert_eq!(resources.tools.len(), TOTAL_ITEMS);
        assert_eq!(resources.paints.len(), TOTAL_WEIGHT_KG);
    }

    #[test]
    fn test_take_out_resources() {
        let mut resources = SharedResources::default();
        let initial_tool_count = resources.tools[0].1;
        resources.take_out_resources(vec!["brush".to_string()]);
        assert_eq!(resources.tools[0].1, initial_tool_count - 1);
    }

    #[test]
    fn test_tool_registry() {
        let resources = Arc::new(Mutex::new(SharedResources::default()));
        let mut registry = ArtistToolRegistry::new(&resources);
        let tools = vec!["brush".to_string(), "palette".to_string()];
        registry.tool_registry(1, tools);
        assert_eq!(registry.artist_tool_preferences.len(), 1);
    }

    #[test]
    fn test_tools_usage() {
        let tools = vec![
            ("brush".to_string(), TOTAL_ITEMS),
            ("palette".to_string(), TOTAL_ITEMS),
        ];
        let (id, selected_tools) = tools_usage(1, &tools);
        assert!(
            selected_tools.len() >= MIN_REQUIRED_TOOLS && selected_tools.len() <= MAX_ALLOWED_TOOLS
        );
    }
}
