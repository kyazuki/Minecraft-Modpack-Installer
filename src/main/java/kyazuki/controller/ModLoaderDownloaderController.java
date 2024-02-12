package kyazuki.controller;

import java.io.IOException;
import java.util.logging.Level;
import java.util.logging.Logger;

import javafx.concurrent.Service;
import javafx.concurrent.Task;
import javafx.concurrent.WorkerStateEvent;
import javafx.event.EventHandler;
import javafx.fxml.FXML;
import kyazuki.App;
import kyazuki.dataclass.Config;

public class ModLoaderDownloaderController {
    @FXML
    public void initialize() {
        Service<Void> service = new Service<>() {
            @Override
            protected Task<Void> createTask() {
                return new Task<>() {
                    @Override
                    protected Void call() throws Exception {
                        Logger logger = App.getLogger();
                        Config config = App.getConfig();
                        if (config.modLoader != null) {
                            // ModLoaderをダウンロードする
                            try {
                                if (config.modLoader.download()) {
                                    logger.info("ModLoader Downloaded.");
                                } else {
                                    logger.info("Skipping download mod loader.");
                                }
                            } catch (IOException e) {
                                logger.log(Level.SEVERE, "Failed to download Mod Loader.", e);
                                throw e;
                            }
                        }
                        return null;
                    }
                };
            }
        };
        service.setOnSucceeded(new EventHandler<WorkerStateEvent>() {
            @Override
            public void handle(WorkerStateEvent event) {
                try {
                    App.setRoot("modsDownloader");
                } catch (IOException e) {
                    App.getLogger().log(Level.SEVERE, "Failed to transit modsDownloader view.", e);
                    App.showAlertAndExit("ビュー遷移に失敗しました。");
                }
            }
        });
        service.setOnFailed(new EventHandler<WorkerStateEvent>() {
            @Override
            public void handle(WorkerStateEvent event) {
                App.showAlertAndExit("ModLoaderのダウンロードに失敗しました。");
            }
        });
        service.start();
    }
}
