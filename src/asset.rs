use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext},
    prelude::*,
    reflect::TypePath,
    utils::{BoxedFuture, HashMap},
};
use futures_lite::AsyncReadExt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, TypePath, Asset, Clone)]
pub struct ItemAsset {
    text: String,
}

#[derive(Debug, Serialize, Deserialize, TypePath, Asset)]
pub struct FolderAsset {
    #[serde(flatten)]
    content: HashMap<String, Box<JsonObject>>,
}

#[derive(Debug, Serialize, Deserialize, TypePath)]
#[serde(tag = "type", rename_all = "kebab-case")]
enum JsonObject {
    Folder(FolderAsset),
    Item(ItemAsset),
}

#[derive(Debug, Serialize, Deserialize, TypePath, Asset)]
pub struct RootAsset(JsonObject);

pub struct JsonExampleLoader;

impl JsonExampleLoader {
    fn visit_node<'a>(&self, key: String, value: &'a mut JsonObject, lc: &'a mut LoadContext) {
        match value {
            JsonObject::Item(ref mut item) => {
                info!("Adding item with key: [{}]", key);
                lc.add_labeled_asset(key, item.clone());
            }
            JsonObject::Folder(folder) => folder.content.drain().for_each(|(ckey, mut obj)| {
                self.visit_node(format!("{}.{}", key, ckey).to_string(), &mut obj, lc)
            }),
        };
    }
}

impl AssetLoader for JsonExampleLoader {
    type Asset = RootAsset;
    type Settings = ();

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, anyhow::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let mut root: JsonObject =
                serde_json::from_slice(&bytes).expect("unable to decode json");
            match root {
                JsonObject::Item(ref mut _item) => {
                    // TODO: This should be the default asset in this case.
                }
                JsonObject::Folder(ref mut folder) => {
                    folder.content.drain().for_each(|(ckey, mut obj)| {
                        self.visit_node(ckey.to_string(), &mut obj, load_context)
                    })
                }
            }
            Ok(RootAsset(root))
        })
    }

    fn extensions(&self) -> &[&str] {
        &["json"]
    }
}
