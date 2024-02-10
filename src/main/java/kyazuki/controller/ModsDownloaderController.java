package kyazuki.controller;

import java.io.IOException;
import java.util.logging.Level;
import java.util.logging.Logger;

import javafx.concurrent.Service;
import javafx.concurrent.Task;
import javafx.concurrent.WorkerStateEvent;
import javafx.event.EventHandler;
import javafx.fxml.FXML;
import javafx.scene.control.Label;
import javafx.scene.control.ProgressBar;
import kyazuki.App;
import kyazuki.dataclass.Config;
import kyazuki.dataclass.Config.CurseForgeMod;

public class ModsDownloaderController {
    @FXML
    private Label modNameLabel;
    @FXML
    private ProgressBar progressBar;

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
                        // Mod群をダウンロードする
                        logger.info("Start downloading mods...");
                        for (int i = 0; i < config.curseForgeMods.length; i++) {
                            CurseForgeMod mod = config.curseForgeMods[i];
                            updateMessage(mod.name);
                            try {
                                if (mod.download()) {
                                    logger.info("\tDownloaded: " + mod.name);
                                } else {
                                    logger.info("\tSkipping download: " + mod.name);
                                }
                                updateProgress(i + 1, config.curseForgeMods.length);
                            } catch (IOException e) {
                                logger.log(Level.SEVERE, "Failed to download mod: " + mod.name, e);
                                throw e;
                            }
                        }
                        return null;
                    }
                };
            }
        };
        modNameLabel.textProperty().bind(service.messageProperty());
        progressBar.progressProperty().bind(service.progressProperty());
        service.setOnSucceeded(new EventHandler<WorkerStateEvent>() {
            @Override
            public void handle(WorkerStateEvent event) {
                try {
                    App.setRoot("profileAppender");
                } catch (IOException e) {
                    App.getLogger().log(Level.SEVERE, "Failed to transit profileAppender view.",
                            e);
                    App.showAlertAndExit("ビュー遷移に失敗しました。");
                }
            }
        });
        service.setOnFailed(new EventHandler<WorkerStateEvent>() {
            @Override
            public void handle(WorkerStateEvent event) {
                App.showAlertAndExit("Modのダウンロードに失敗しました。");
            }
        });
        service.start();
    }
}
